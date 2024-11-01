use std::io::{self, Write};

use super::terminal::Ansi;

#[derive(Debug)]
pub struct State {
    pub board: [char; 9],
    pub board_pos: (u16, u16),
    pub cursor_pos: (u16, u16),
    pub active: bool, 
}

pub fn new() -> State {
    State {
        board: [' '; 9],
        board_pos: (1, 1),
        cursor_pos: (2, 2),
        active: true,
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
    super::terminal::print_debug(state);
    print!(
        "{}",
        Ansi::MoveCursor(state.cursor_pos.0, state.cursor_pos.1)
    );
    io::stdout().flush()?;
    Ok(())
}

pub fn attempt_placing(state: &mut State, symbol: char) {
    let valid_pos = [(3,2), (7, 2), (11, 2), (3,4), (7, 4), (11, 4), (3,6), (7, 6), (11, 6)];

    if let Some(placement_index) = valid_pos.iter().position(|pos| pos == &state.cursor_pos) {
        state.board[placement_index] = symbol;
    };
}

pub fn check_state(state: &mut State) {
    if !state.board.contains(&' ') {
        state.active = false;
    }
}