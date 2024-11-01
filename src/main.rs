use std::{thread, time};

mod game;
mod terminal;

fn main() -> anyhow::Result<()> {
    terminal::init();

    let mut game_state = game::new();

    loop {
        game::render(&game_state)?;
        terminal::read_input(&mut game_state)?;
        thread::sleep(time::Duration::from_millis(33));
    }

    Ok(())
}
