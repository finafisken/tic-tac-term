use std::io::{self, Write};

use crate::terminal;

#[derive(Debug)]
pub struct Game {
    state: State,
    pub player: Player,
    pub mode: Mode,
    pub board_pos: (u16, u16),
    pub cursor_pos: (u16, u16),
    pub free_cursor: bool,
    pub symbol_slots: [(u16, u16); 9],
}

impl Game {
    pub fn new(mode: Mode) -> Self {
        Game {
            player: Player::O,
            mode,
            state: State {
                board: [' '; 9],
                active: true,
                current_player: Player::O,
                winner: None,
            },
            symbol_slots: [
                (3, 2),
                (7, 2),
                (11, 2),
                (3, 4),
                (7, 4),
                (11, 4),
                (3, 6),
                (7, 6),
                (11, 6),
            ],
            board_pos: (1, 1),
            cursor_pos: (3, 2),
            free_cursor: false,
        }
    }

    pub fn check_state(&mut self) {
        self.state.check_status();
    }

    pub fn draw_board(&self) {
        print!(
            "{}",
            terminal::Ansi::MoveCursor(self.board_pos.0, self.board_pos.1)
        );
        println!("┌───┬───┬───┐");
        println!(
            "│ {} │ {} │ {} │",
            self.state.board[0], self.state.board[1], self.state.board[2]
        );
        println!("├───┼───┼───┤");
        println!(
            "│ {} │ {} │ {} │",
            self.state.board[3], self.state.board[4], self.state.board[5]
        );
        println!("├───┼───┼───┤");
        println!(
            "│ {} │ {} │ {} │",
            self.state.board[6], self.state.board[7], self.state.board[8]
        );
        println!("└───┴───┴───┘");
    }

    pub fn render(&self) -> anyhow::Result<()> {
        print!("{}", terminal::Ansi::ClearScreen);
        self.draw_board();
        super::terminal::print_debug(self);
        print!(
            "{}",
            terminal::Ansi::MoveCursor(self.cursor_pos.0, self.cursor_pos.1)
        );
        io::stdout().flush()?;
        Ok(())
    }

    pub fn attempt_placing(&mut self, symbol: char) {
        if let Some(placement_index) = self.symbol_slots.iter().position(|pos| pos == &self.cursor_pos) {
            if self.state.board[placement_index] == ' ' && self.state.current_player == symbol.into() && self.state.active {
                self.state.board[placement_index] = symbol;
                self.state.current_player = self.state.current_player.end_turn();
            }
        };
    }

    pub fn get_current_player(&self) -> &Player {
        &self.state.current_player
    }

    pub fn restart(&mut self) {
        self.state.restart();
    }
}

#[derive(Debug)]
pub struct State {
    pub board: [char; 9],
    pub active: bool,
    pub current_player: Player,
    pub winner: Option<Player>,
}

impl State {
    pub fn restart(&mut self) {
        self.board = [' '; 9];
        self.active = true;
        self.current_player = Player::O;
        self.winner = None;
    }

    pub fn check_status(&mut self) {
        let rows_result = self.check_rows();
        let cols_result = self.check_cols();
        let diagonal_result = self.check_diagonal();
    
        self.winner = rows_result.or(cols_result).or(diagonal_result);
    
        if !self.board.contains(&' ') || self.winner.is_some() {
            self.active = false;
        }
    }

    fn check_rows(&mut self) -> Option<Player> {
        self
            .board
            .chunks(3)
            .find(|row| row[0] == row[1] && row[1] == row[2] && row[0] != ' ')
            .map(|c_arr| c_arr[0].into())
    }
    
    fn check_cols(&mut self) -> Option<Player> {
        for col in 0..3 {
            if self.board[col] == self.board[col + 3]
                && self.board[col + 3] == self.board[col + 6]
                && self.board[col] != ' '
            {
                return Some(self.board[col].into());
            }
        }
        None
    }
    
    fn check_diagonal(&mut self) -> Option<Player> {
        let board = self.board;
    
        if board[0] == board[4] && board[4] == board[8] && board[4] != ' ' {
            return Some(board[4].into())
        }
    
        if board[2] == board[4] && board[4] == board[6] && board[4] != ' ' {
            return Some(board[4].into())
        }
    
        None
    }
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Local,
    Network,
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

impl From<&Player> for char {
    fn from(value: &Player) -> Self {
        match value {
            Player::O => 'O',
            Player::X => 'X',
        }
    }
}
