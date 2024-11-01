use std::io::{self, Write};

use super::terminal::Ansi;

#[derive(Debug)]
pub struct State {
    pub board: [char; 9],
    pub board_pos: (u16, u16),
    pub cursor_pos: (u16, u16),
}

pub fn new() -> State {
    State {
        board: [' '; 9],
        board_pos: (1, 1),
        cursor_pos: (2, 2),
    }
}

fn draw_board(state: &State) {
    print!("{}", Ansi::MoveCursor(state.board_pos.0, state.board_pos.1));
    println!("┌───┬───┬───┐");
    println!(
        "│ {} │ {} │ {} │",
        state.board[0], state.board[1], state.board[2]
    );
    println!("├───┼───┼───┤");
    println!(
        "│ {} │ {} │ {} │",
        state.board[3], state.board[4], state.board[5]
    );
    println!("├───┼───┼───┤");
    println!(
        "│ {} │ {} │ {} │",
        state.board[6], state.board[7], state.board[8]
    );
    println!("└───┴───┴───┘");
}

pub fn render(state: &State) -> anyhow::Result<()> {
    print!("{}", Ansi::ClearScreen);
    draw_board(state);
    print!(
        "{}",
        Ansi::MoveCursor(state.cursor_pos.0, state.cursor_pos.1)
    );
    io::stdout().flush()?;
    Ok(())
}
