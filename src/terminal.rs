use core::fmt;
use libc::{
    c_int, ioctl, signal, tcgetattr, tcsetattr, termios, winsize, ECHO, ICANON, SIGINT, SIGTERM,
    STDOUT_FILENO, TCSANOW, TIOCGWINSZ,
};
use std::{
    cmp,
    io::{self, Write},
    mem,
    sync::{mpsc, Mutex, OnceLock}, time::Duration,
};

use super::game;

pub enum Ansi {
    HideCursor,           // "\x1B[?25l"
    ShowCursor,           // "\x1B[?25h"
    ClearScreen,          //  "\x1B[2J"
    MoveCursor(u16, u16), // "\x1B[%d;%dH" %d num
}

impl fmt::Display for Ansi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ansi::HideCursor => write!(f, "\x1B[?25l"),
            Ansi::ShowCursor => write!(f, "\x1B[?25h"),
            Ansi::ClearScreen => write!(f, "\x1B[2J"),
            Ansi::MoveCursor(x, y) => write!(f, "\x1B[{};{}H", y, x),
        }
    }
}

static ORIGINAL_TERM: OnceLock<Mutex<termios>> = OnceLock::new();

pub fn init() {
    enable_raw_mode();
    unsafe {
        signal(SIGINT, handle_signal as usize);
        signal(SIGTERM, handle_signal as usize);
    }
}

// enable raw mode so we dont have to wait for enter press
fn enable_raw_mode() {
    let mut term = unsafe { mem::zeroed() };
    unsafe {
        tcgetattr(0, &mut term);

        // save original attributes to restore later
        let original_term = term;
        ORIGINAL_TERM.get_or_init(|| Mutex::new(original_term));

        // turn off canonical mode and echo
        term.c_lflag &= !(ICANON | ECHO);
        tcsetattr(0, TCSANOW, &term);
    }
}

pub fn disable_raw_mode() {
    if let Some(lock) = ORIGINAL_TERM.get() {
        unsafe {
            if let Ok(term) = lock.lock() {
                tcsetattr(0, TCSANOW, &*term);
            }
        }
    }
}

pub fn get_size() -> (u16, u16) {
    let mut size: winsize = unsafe { std::mem::zeroed() };
    unsafe {
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size);
    }

    (size.ws_col, size.ws_row)
}

pub extern "C" fn handle_signal(_signal: c_int) {
    restore_and_exit();
}

fn restore_and_exit() {
    // clean up and restore cursor
    print!("{}", Ansi::ClearScreen);
    print!("{}", Ansi::ShowCursor);
    print!("{}", Ansi::MoveCursor(1, 1));
    std::io::stdout().flush().unwrap();

    disable_raw_mode();

    std::process::exit(0)
}

fn move_cursor(game: &mut game::Game, term_rx: &mpsc::Receiver<u8>) {
    let Ok(first_byte) = term_rx.recv_timeout(Duration::from_millis(10)) else {
        return;
    };

    if first_byte != b'[' {
        return;
    }

    let Ok(second_byte) = term_rx.recv_timeout(Duration::from_millis(10)) else {
        return;
    };

    let (current_x, current_y) = game.cursor_pos;
    let (max_x, max_y) = get_size();

    if game.free_cursor {
        match [first_byte, second_byte] {
            [b'[', b'A'] => game.cursor_pos = (current_x, cmp::max(current_y - 1, 1)),
            [b'[', b'B'] => game.cursor_pos = (current_x, cmp::min(current_y + 1, max_y)),
            [b'[', b'C'] => game.cursor_pos = (cmp::min(current_x + 1, max_x), current_y),
            [b'[', b'D'] => game.cursor_pos = (cmp::max(current_x - 1, 1), current_y),
            _ => (),
        }
    } else if game.symbol_slots.contains(&game.cursor_pos) {
        match [first_byte, second_byte] {
            [b'[', b'A'] => game.cursor_pos = (current_x, cmp::max(current_y - 2, 2)),
            [b'[', b'B'] => game.cursor_pos = (current_x, cmp::min(current_y + 2, 6)),
            [b'[', b'C'] => game.cursor_pos = (cmp::min(current_x + 4, 11), current_y),
            [b'[', b'D'] => game.cursor_pos = (cmp::max(current_x.saturating_sub(4), 3), current_y),
            _ => (),
        }
    }
}

pub fn process_input(game: &mut game::Game, term_rx: &mpsc::Receiver<u8>) -> anyhow::Result<()> {
    match term_rx.recv_timeout(Duration::from_millis(33))? {
        b'q' => restore_and_exit(),
        b's' => println!("{}", Ansi::ShowCursor),
        b'h' => println!("{}", Ansi::HideCursor),
        b'f' => game.free_cursor = !game.free_cursor,
        b'r' => game.restart(),
        b'x' => game.attempt_placing('X'),
        b'o' => game.attempt_placing('O'),
        b' ' => game.attempt_placing(char::from(game.get_current_player())),
        b'\x1B' => move_cursor(game, term_rx),
        _ => (),
    }

    Ok(())
}

pub fn print_debug<T: fmt::Debug>(data: T) {
    print!("{}", Ansi::MoveCursor(1, get_size().1 - 4));
    println!("{:?}", data);
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use crate::game::{Game, Mode, Player};

    #[test]
    fn test_ansi_format() {
        assert_eq!(format!("{}", Ansi::HideCursor), "\x1B[?25l");
        assert_eq!(format!("{}", Ansi::ShowCursor), "\x1B[?25h");
        assert_eq!(format!("{}", Ansi::ClearScreen), "\x1B[2J");
        assert_eq!(format!("{}", Ansi::MoveCursor(10, 20)), "\x1B[20;10H");
    }

    #[test]
    fn test_move_cursor_free_mode() {
        let (tx, rx) = mpsc::channel();
        let mut game = Game::new(Mode::Local, true);
        game.free_cursor = true;
        game.cursor_pos = (5, 5);

        // Test arrow up
        tx.send(b'[').unwrap();
        tx.send(b'A').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (5, 4));

        // Test arrow down
        tx.send(b'[').unwrap();
        tx.send(b'B').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (5, 5));

        // Test arrow right
        tx.send(b'[').unwrap();
        tx.send(b'C').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (6, 5));

        // Test arrow left
        tx.send(b'[').unwrap();
        tx.send(b'D').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (5, 5));

        // Test boundary conditions
        game.cursor_pos = (1, 1);
        tx.send(b'[').unwrap();
        tx.send(b'D').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (1, 1)); // Should not go below 1

        tx.send(b'[').unwrap();
        tx.send(b'A').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (1, 1)); // Should not go below 1
    }

    #[test]
    fn test_move_cursor_fixed_mode() {
        let (tx, rx) = mpsc::channel();
        let mut game = Game::new(Mode::Local, true);
        
        // Set up symbol slots
        game.symbol_slots = [
            (3, 2), (7, 2), (11, 2),
            (3, 4), (7, 4), (11, 4),
            (3, 6), (7, 6), (11, 6),
        ];
        
        game.free_cursor = false;
        game.cursor_pos = (7, 4); // Middle slot
        
        // Test arrow up
        tx.send(b'[').unwrap();
        tx.send(b'A').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (7, 2));

        // Test arrow down
        tx.send(b'[').unwrap();
        tx.send(b'B').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (7, 4));
        
        // Test arrow right
        tx.send(b'[').unwrap();
        tx.send(b'C').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (11, 4));
        
        // Test arrow left
        tx.send(b'[').unwrap();
        tx.send(b'D').unwrap();
        move_cursor(&mut game, &rx);
        assert_eq!(game.cursor_pos, (7, 4));
    }
    
    #[test]
    fn test_process_input() {
        let (tx, rx) = mpsc::channel();
        let mut game = Game::new(Mode::Local, true);
        
        // Test place X
        tx.send(b'x').unwrap();
        let result = process_input(&mut game, &rx);
        assert!(result.is_ok());
        
        // Test toggle free cursor
        let current_state = game.free_cursor;
        tx.send(b'f').unwrap();
        let result = process_input(&mut game, &rx);
        assert!(result.is_ok());
        assert_eq!(game.free_cursor, !current_state);
        
        // Test placing at current position
        game.cursor_pos = game.symbol_slots[0];
        game.state.current_player = Player::O;
        tx.send(b' ').unwrap();
        let result = process_input(&mut game, &rx);
        assert!(result.is_ok());
        // The board should now have an O at position 0
        assert_eq!(game.state.board[0], 'O');
        
        // Test arrow key input (first part of sequence)
        tx.send(b'\x1B').unwrap();
        // Need to send the follow-up bytes quickly
        tx.send(b'[').unwrap();
        tx.send(b'B').unwrap();
        let result = process_input(&mut game, &rx);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_input() {
        let (tx, rx) = mpsc::channel();
        let mut game = Game::new(Mode::Local, true);
        
        // Test timeout (no input)
        let result = process_input(&mut game, &rx);
        assert!(result.is_err());
        
        // Test incomplete escape sequence
        tx.send(b'\x1B').unwrap();
        let result = process_input(&mut game, &rx);
        assert!(result.is_ok()); // Should not crash, but might timeout
    }
    
    // This is a special test - we can't fully test the function
    // but we can verify it doesn't crash when called
    #[test]
    fn test_print_debug_doesnt_crash() {
        // This just verifies the function doesn't panic
        print_debug("Test debug message");
    }
}
