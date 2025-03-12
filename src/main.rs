use std::{env, thread, time};

mod game;
mod terminal;
mod network;

fn main() -> anyhow::Result<()> {
    let (game_mode, addr, is_host) =  parse_args();
    terminal::init();

    // if game_mode == game::Mode::Network {
        let (mut net_read, mut net_write) = network::connect(&addr)?;
    // }

    loop {
        network::read_stream(&mut net_read)?;
        thread::sleep(time::Duration::from_millis(2000));
        let data: Vec<u8> = b"you called".to_vec();
        network::write_stream(&mut net_write, data)?;
    }

    // let mut game = game::Game::new(game_mode, is_host);

    // loop {
    //     game.render()?;
    //     // read from stdin if player turn or local game otherwise read from network
    //     if game.get_current_player() == &game.player || game.mode == game::Mode::Local {
    //         terminal::read_input(&mut game)?;
    //     } else {
    //         // net_read.read(buf)
    //     }
    //     game.check_state();
    //     // send state if mp
    //     // net_write.write(&game.state);
    //     thread::sleep(time::Duration::from_millis(33));
    // }

    Ok(())
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
