use std::{io::{self, Write}, thread, time};
use terminal::Ansi;

mod terminal;
mod game;

fn draw_board(state: &game::State) {
    print!("{}", Ansi::MoveCursor(state.board_pos.0, state.board_pos.1));
    println!("┌───┬───┬───┐");
    println!("│ {} │ {} │ {} │", state.board[0], state.board[1], state.board[2]);
    println!("├───┼───┼───┤");
    println!("│ {} │ {} │ {} │", state.board[3], state.board[4], state.board[5]);
    println!("├───┼───┼───┤");
    println!("│ {} │ {} │ {} │", state.board[6], state.board[7], state.board[8]);
    println!("└───┴───┴───┘");
}

fn render(state: &game::State) -> anyhow::Result<()> {
    print!("{}", Ansi::ClearScreen);
    draw_board(state);
    print!("{}", Ansi::MoveCursor(state.cursor_pos.0, state.cursor_pos.1));
    io::stdout().flush()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    terminal::init();

    let mut game_state = game::new();

    loop {
        render(&game_state)?;
        terminal::read_input()?;
        thread::sleep(time::Duration::from_millis(33));
    }

    Ok(())
}
