use std::{env, sync::mpsc, thread, time, io::{self, Read},};
use game::{Game, Mode, State};
use network::NetState;

mod game;
mod network;
mod terminal;

// X, , , , , ,O, , ###X

fn main() -> anyhow::Result<()> {
    let (game_mode, addr, is_host) = parse_args();
    terminal::init();

    let (game_tx, game_rx) = mpsc::channel::<String>();
    let (net_tx, net_rx) = mpsc::channel::<String>();
    let (term_tx, term_rx) = mpsc::channel::<u8>();

    if game_mode == Mode::Network {
        let (mut net_read, mut net_write) = network::connect(&addr, is_host)?;
        thread::spawn(move || {
            loop {
                let incoming = network::read_stream(&mut net_read).unwrap();
                net_tx.send(incoming);
                thread::sleep(time::Duration::from_millis(33));
            }
        });

        thread::spawn(move || {
            loop {
                if let Ok(data) = game_rx.recv() {
                    let game_state = data.into_bytes().clone();
                    network::write_stream(&mut net_write, game_state).unwrap();
                    thread::sleep(time::Duration::from_millis(33));
                }
            }
        });
    }

    thread::spawn(move || {
        loop {
            let mut buffer = [0; 1];
            io::stdin().read_exact(&mut buffer);

            term_tx.send(buffer[0]);
        }
    });

    let mut game = Game::new(game_mode, is_host);

    loop {
        // if is_host {game.render()?} else {println!("{:?}", game)};
        // println!("{:?}", game);
        game.render()?;

        terminal::process_input(&mut game, &term_rx);

        if let Ok(recieved) = net_rx.recv_timeout(time::Duration::from_millis(33)) {
            if recieved.contains("###") {
                let recieved_state: State = recieved.into();
                if recieved_state.round > game.state.round {
                    let validation_result = game.validate(recieved_state);
                    let reply = match validation_result {
                        Ok(_) => format!("{:?}", network::MessageType::Accepted),
                        Err(_) => format!("{:?}", network::MessageType::Rejected),
                    };
    
                    game_tx.send(reply)?;
                    game.net_state = NetState::Active;
                }
            } else if recieved.contains("Accepted") {
                // set netstate
                // CHECK ACTUAL RESPONSE
                game.net_state = NetState::Waiting;
               
            }
            // game.net_state = network::NetState::Active;
        }
        game.check_state();

        if game.mode == Mode::Network && game.net_state == NetState::Active {
            game_tx.send(game.state.to_string())?;
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
