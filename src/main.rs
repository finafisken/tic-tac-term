use std::{env, thread, time};

mod game;
mod terminal;
mod network;

fn main() -> anyhow::Result<()> {
    let (game_mode, addr, is_host) =  parse_args();
    terminal::init();

    let (read, write) = network::connect(&addr)?;

    let mut game = game::Game::new(game_mode);

    loop {
        game.render()?;
        // if mp read from listener, sp/turn read from input
        terminal::read_input(&mut game)?;
        game.check_state();
        // send state if mp
        thread::sleep(time::Duration::from_millis(33));
    }




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

// fn run_game() -> anyhow::Result<()> {
//     let mut game = game::Game::new(game::Mode::Local);

//     loop {
//         game.render()?;
//         // if mp read from listener, sp/turn read from input
//         terminal::read_input(&mut game)?;
//         game.check_state();
//         // send state if mp
//         thread::sleep(time::Duration::from_millis(33));
//     }
// }
