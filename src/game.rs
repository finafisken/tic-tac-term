use std::io::{self, Write};

use crate::terminal;

#[derive(Debug)]
pub struct State {
    pub board: [char; 9],
    pub board_pos: (u16, u16),
    pub cursor_pos: (u16, u16),
    pub active: bool,
    pub current_player: Player,
    pub winner: Option<Player>,
}

pub fn new() -> State {
    State {
        board: [' '; 9],
        board_pos: (1, 1),
        cursor_pos: (2, 2),
        active: true,
        current_player: Player::O,
        winner: None,
    }
}

#[derive(Debug, PartialEq)]
pub enum Player {
    X,
    O,
}

impl Player {
    fn end_turn(&self) -> Player {
        match self {
            Player::O => Player::X,
            Player::X => Player::O,
        }
    }
}

impl From<char> for Player {
    fn from(value: char) -> Self {
        match value.to_ascii_uppercase() {
            'O' => Player::O,
            'X' => Player::X,
            _ => panic!("Unknown player"),
        }
    }
}

fn draw_board(state: &State) {
    print!(
        "{}",
        terminal::Ansi::MoveCursor(state.board_pos.0, state.board_pos.1)
    );
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
    print!("{}", terminal::Ansi::ClearScreen);
    draw_board(state);
    super::terminal::print_debug(state);
    print!(
        "{}",
        terminal::Ansi::MoveCursor(state.cursor_pos.0, state.cursor_pos.1)
    );
    io::stdout().flush()?;
    Ok(())
}

pub fn attempt_placing(state: &mut State, symbol: char) {
    let valid_pos = [
        (3, 2),
        (7, 2),
        (11, 2),
        (3, 4),
        (7, 4),
        (11, 4),
        (3, 6),
        (7, 6),
        (11, 6),
    ];

    if let Some(placement_index) = valid_pos.iter().position(|pos| pos == &state.cursor_pos) {
        if state.board[placement_index] == ' ' && state.current_player == symbol.into() {
            state.board[placement_index] = symbol;
            state.current_player = state.current_player.end_turn();
        }
    };
}

pub fn check_state(state: &mut State) {
    state.winner = check_rows(state);
    state.winner = check_cols(state);

    if !state.board.contains(&' ') || state.winner.is_some() {
        state.active = false;
    }
}

fn check_rows(state: &mut State) -> Option<Player> {
    state
        .board
        .chunks(3)
        .find(|row| row[0] == row[1] && row[1] == row[2] && row[0] != ' ')
        .map(|c_arr| c_arr[0].into())
}

fn check_cols(state: &mut State) -> Option<Player> {
    for col in 0..3 {
        if state.board[col] == state.board[col + 3]
            && state.board[col + 3] == state.board[col + 6]
            && state.board[col] != ' '
        {
            return Some(state.board[col].into());
        }
    }
    None
}
