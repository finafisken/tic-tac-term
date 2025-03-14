use std::{env, sync::mpsc, thread, time::{self, Instant}};

mod game;
mod terminal;
mod network;

// X, , , , , ,O, , ###X

fn main() -> anyhow::Result<()> {
    let (game_mode, addr, is_host) =  parse_args();
    terminal::init();

    let (game_tx, game_rx) = mpsc::channel::<String>();
    let (net_tx, net_rx) = mpsc::channel::<String>();
    let (term_tx, term_rx) = mpsc::channel::<String>();
    let mut last_send_ts = Instant::now();

    if game_mode == game::Mode::Network {
        let (mut net_read, mut net_write) = network::connect(&addr, is_host)?;
        thread::spawn(move || {
            loop {
                let incoming = network::read_stream(&mut net_read).unwrap();
                // terminal::print_debug(&incoming);
                // println!("{}", incoming);
                net_tx.send(incoming).unwrap();
            }
        });

        thread::spawn(move || {
            loop {
                let data = game_rx.recv().unwrap().into_bytes().clone();
                network::write_stream(&mut net_write, data).unwrap();
            }
        });
    }

    let mut game = game::Game::new(game_mode, is_host);

    loop {
        // if is_host {game.render()?} else {println!("{:?}", game)};
        game.render()?;

        terminal::read_input(&mut game)?;
        
        if let Ok(recieved) = net_rx.recv_timeout(time::Duration::from_millis(10)) {
            game.validate(recieved.into())?;
        }
        game.check_state();
        // send state every 2 seconds as heartbeat and recovery
        if game.mode == game::Mode::Network && &game.player != game.get_current_player() && last_send_ts.elapsed() >= time::Duration::from_secs(2) {
            game_tx.send(game.state.to_string())?;
            last_send_ts = Instant::now();
        }
        thread::sleep(time::Duration::from_millis(33));
    }
}

fn parse_args() -> (game::Mode, String, bool) {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        return (game::Mode::Local, String::default(), false)
    }

    let addr = args.get(2).expect("no address provided").clone();
    let is_host = args.get(1).expect("missing argument") == &String::from("host");

    (game::Mode::Network, addr, is_host)
}
