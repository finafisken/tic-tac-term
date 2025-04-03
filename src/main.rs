use std::{env, sync::mpsc, thread, time, io::{self, Read},};
use game::{Game, Mode, State};
use network::{Message, MessageType, NetState};

mod game;
mod network;
mod terminal;

fn main() -> anyhow::Result<()> {
    let (game_mode, addr, is_host) = parse_args();
    terminal::init();

    let (game_tx, game_rx) = mpsc::channel::<Message>();
    let (net_tx, net_rx) = mpsc::channel::<Message>();
    let (term_tx, term_rx) = mpsc::channel::<u8>();

    if game_mode == Mode::Network {
        let (mut net_read, mut net_write) = network::connect(&addr, is_host)?;
        thread::spawn(move || {
            loop {
                let incoming = network::read_stream(&mut net_read).unwrap();
                net_tx.send(incoming).unwrap();
                thread::sleep(time::Duration::from_millis(33));
            }
        });

        thread::spawn(move || {
            loop {
                if let Ok(msg) = game_rx.recv() {
                    network::write_stream(&mut net_write, msg.into()).unwrap();
                    thread::sleep(time::Duration::from_millis(33));
                }
            }
        });
    }

    thread::spawn(move || {
        loop {
            let mut buffer = [0; 1];
            io::stdin().read_exact(&mut buffer).unwrap();
            term_tx.send(buffer[0]).unwrap();
        }
    });

    let mut game = Game::new(game_mode, is_host);

    loop {
        game.render()?;

        let _ = terminal::process_input(&mut game, &term_rx);

        if let Ok(recieved) = net_rx.recv_timeout(time::Duration::from_millis(33)) {
            match recieved.message_type {
                MessageType::Accepted => game.net_state = NetState::Waiting,
                MessageType::Rejected => game.net_state = NetState::Active,
                MessageType::Payload => {
                    let recieved_state: State = recieved.payload.as_slice().try_into()?;
                    if recieved_state.round > game.state.round {
                        let validation_result = game.validate(recieved_state);
                        let reply = match validation_result {
                            Ok(_) => MessageType::Accepted,
                            Err(_) => MessageType::Rejected,
                        };
        
                        game_tx.send(Message { message_type: reply, payload_size: 0, payload: Vec::new() })?;
                        game.net_state = NetState::Active;
                    }
                },
            }
        }

        game.check_state();

        if game.mode == Mode::Network && game.net_state == NetState::Active {
            let payload: Vec<u8> = (&game.state).into();
            let message = Message {
                message_type: MessageType::Payload,
                payload_size: payload.len() as u16,
                payload
            };

            game_tx.send(message)?;
        }

        thread::sleep(time::Duration::from_millis(33));
    }
}

fn parse_args() -> (Mode, String, bool) {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        return (Mode::Local, String::default(), false);
    }

    let addr = args.get(2).expect("no address provided").clone();
    let is_host = args.get(1).expect("missing argument") == &String::from("host");

    (Mode::Network, addr, is_host)
}
