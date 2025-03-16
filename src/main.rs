use std::{env, sync::mpsc, thread, time};
use game::{Game, Mode};
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

    if game_mode == Mode::Network {
        let (mut net_read, mut net_write) = network::connect(&addr, is_host)?;
        thread::spawn(move || {
            loop {
                let incoming = network::read_stream(&mut net_read).unwrap();
                // terminal::print_debug(&incoming);
                // println!("{}", incoming);
                net_tx.send(incoming).unwrap();
                // thread::sleep(time::Duration::from_millis(33));
            }
        });

        thread::spawn(move || loop {
            let data = game_rx.recv().unwrap().into_bytes().clone();
            network::write_stream(&mut net_write, data).unwrap();
            // thread::sleep(time::Duration::from_millis(33));
        });
    }

    let mut game = Game::new(game_mode, is_host);

    loop {
        // if is_host {game.render()?} else {println!("{:?}", game)};
        game.render()?;

        terminal::read_input(&mut game)?;

        if let Ok(recieved) = net_rx.recv_timeout(time::Duration::from_millis(33)) {
            if recieved.contains("###") && game.net_state == NetState::Waiting  {
                let validation_result = game.validate(recieved.into());
                let reply = match validation_result {
                    Ok(_) => format!("{:?}", network::MessageType::Accepted),
                    Err(_) => format!("{:?}", network::MessageType::Rejected),
                };

                game_tx.send(reply)?;
                game.net_state = NetState::Active;
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
