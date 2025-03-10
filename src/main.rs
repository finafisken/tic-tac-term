use std::{thread, time};

mod game;
mod terminal;

fn main() -> anyhow::Result<()> {
    terminal::init();

    let mut game = game::Game::new(game::Mode::Local);

    loop {
        game.render()?;
        terminal::read_input(&mut game)?;
        game.check_state();
        thread::sleep(time::Duration::from_millis(33));
    }
}
