use std::{
    fmt,
    io::{self, Write},
};

use anyhow::anyhow;

use crate::{
    network::NetState,
    terminal,
};

#[derive(Debug)]
pub struct Game {
    pub state: State,
    pub player: Player,
    pub net_state: NetState,
    pub mode: Mode,
    pub board_pos: (u16, u16),
    pub cursor_pos: (u16, u16),
    pub free_cursor: bool,
    pub symbol_slots: [(u16, u16); 9],
}

impl Game {
    pub fn new(mode: Mode, is_host: bool) -> Self {
        let mut player = Player::O;
        let mut net_state = NetState::Active;
        if mode == Mode::Network && !is_host {
            player = Player::X;
            net_state = NetState::Waiting;
        }

        Game {
            player,
            mode,
            net_state,
            state: State {
                board: [' '; 9],
                round: 0,
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
        if self.mode == Mode::Network && self.player != symbol.into() {
            // network mode and not players turn
            return;
        }

        if let Some(placement_index) = self
            .symbol_slots
            .iter()
            .position(|pos| pos == &self.cursor_pos)
        {
            if self.state.board[placement_index] == ' '
                && self.state.current_player == symbol.into()
                && self.state.active
            {
                self.state.board[placement_index] = symbol;
                self.state.round += 1;
                self.state.current_player = self.state.current_player.end_turn();
            }
        };
    }

    pub fn validate(&mut self, potential_state: State) -> anyhow::Result<()> {
        // TODO validation logic
        self.state = potential_state;
        Ok(())
    }

    pub fn get_current_player(&self) -> &Player {
        &self.state.current_player
    }

    pub fn restart(&mut self) {
        match self.mode {
            Mode::Local => self.state.restart(),
            Mode::Network => (),
        }
    }
}

#[derive(Debug)]
pub struct State {
    pub board: [char; 9],
    pub round: u8,
    pub active: bool,
    pub current_player: Player,
    pub winner: Option<Player>,
}

impl State {
    pub fn restart(&mut self) {
        self.board = [' '; 9];
        self.round = 0;
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
        self.board
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
            return Some(board[4].into());
        }

        if board[2] == board[4] && board[4] == board[6] && board[4] != ' ' {
            return Some(board[4].into());
        }

        None
    }
}

/// Binary format (11 bytes):
/// - Bytes 0-8: Board state ('X', 'O', or ' ' for each cell)
/// - Byte 9: Round count
/// - Byte 10: Flag byte
///   - Bit 0: Current player (0 = X, 1 = O)
///   - Bit 1: Game active (0 = inactive, 1 = active)
///   - Bit 2: Has winner (0 = no, 1 = yes)
///   - Bit 3: Winner type (0 = X, 1 = O) if bit 2 is set
impl TryFrom<&[u8]> for State {
    fn try_from(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != 11 {
            return Err(anyhow!("Full state can only be deserialized from 11 bytes"))
        }
        // board is 9 bytes (no need for full char (4bytes), can only have 3 values)
        let board_bytes = &bytes[0..9];
        let board: [char; 9] = board_bytes
            .iter()
            .map(|b| match b {
                b'X' => 'X',
                b'O' => 'O',
                _ => ' ',
            })
            .collect::<Vec<char>>()
            .try_into()
            .expect("Failed to convert board bytes");

        // round count is single u8
        let round = bytes[9];

        // current_player = 1bit
        // winner = 2bit (some/none + player)
        // active = 1bit bool
        let flags_byte = bytes[10];

        // extract flags 
        // TODO add more comments about bit ops
        let current_player = if (flags_byte & 1) == 0 { Player::X } else { Player::O };
        let active = (flags_byte & (1 << 1)) != 0;
        let has_winner = (flags_byte & (1 << 2)) != 0;
        let winner = if has_winner {
            Some(if (flags_byte & (1 << 3)) != 0 { Player::O } else { Player::X })
        } else {
            None
        };

        // full state from 11byte
        Ok(State {
            board,
            round,
            active,
            current_player,
            winner,
        })
    }

    type Error = anyhow::Error;
}

impl From<&State> for Vec<u8> {
    fn from(state: &State) -> Self {
        let mut bytes: Vec<u8> = Vec::with_capacity(11);

        // board 9 bytes
        for c in state.board {
            let byte = match c {
                'X' => b'X',
                'O' => b'O',
                _ => b' ',
            };
            bytes.push(byte)
        };

        // round count 1 byte (u8)
        bytes.push(state.round);

        // pack flags into a single byte
        let mut flags_byte: u8 = 0;
        
        // current player (bit 0)
        if state.current_player == Player::O {
            flags_byte |= 1;
        }
        
        // active state (bit 1)
        if state.active {
            flags_byte |= 1 << 1;
        }
        
        // has winner (bit 2)
        if state.winner.is_some() {
            flags_byte |= 1 << 2;
            
            // winner player (bit 3)
            if let Some(Player::O) = state.winner {
                flags_byte |= 1 << 3;
            }
        }
        
        // flags byte (1 byte)
        bytes.push(flags_byte);

        bytes
    }
}

impl From<String> for State {
    fn from(value: String) -> Self {
        let x: Vec<&str> = value.split("###").collect();
        let board: [char; 9] = x
            .first()
            .expect("No board data")
            .split(',')
            .map(|s| s.chars().next().unwrap_or(' '))
            .collect::<Vec<char>>()
            .try_into()
            .expect("failed to convert to char");
        let current_player: Player = x
            .get(1)
            .expect("no current player")
            .chars()
            .next()
            .expect("cant parse char")
            .into();
        let round = x.get(2).map(|r| r.parse::<u8>().unwrap()).unwrap();
        let active = x.get(3).map(|s| s.parse::<bool>().unwrap()).unwrap();
        let winner = x.get(4).and_then(|s| s.chars().next()).map(Player::from);

        State {
            board,
            round,
            active,
            current_player,
            winner,
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board = self
            .board
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let curr_player: char = (&self.current_player).into();
        let winner = self.winner.as_ref().map(char::from).unwrap_or(' ');
        write!(
            f,
            "{}###{}###{}###{}###{}",
            board, curr_player, self.round, self.active, winner
        )
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
