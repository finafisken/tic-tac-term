use core::fmt;
use libc::{
    c_int, ioctl, signal, tcgetattr, tcsetattr, termios, winsize, ECHO, ICANON, SIGINT, SIGTERM,
    STDOUT_FILENO, TCSANOW, TIOCGWINSZ,
};
use std::{
    cmp,
    io::{self, Read, Write},
    mem,
    sync::Mutex,
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

static mut ORIGINAL_TERM: Mutex<termios> = Mutex::new(unsafe { mem::zeroed() });

pub fn init() {
    enable_raw_mode();
    // print!("{}", Ansi::HideCursor);
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
        *ORIGINAL_TERM.lock().unwrap() = original_term;

        // turn off canonical mode and echo
        term.c_lflag &= !(ICANON | ECHO);
        tcsetattr(0, TCSANOW, &term);
    }
}

pub fn disable_raw_mode() {
    unsafe {
        let original_term = *ORIGINAL_TERM.lock().unwrap();
        tcsetattr(0, TCSANOW, &original_term);
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

fn move_cursor(state: &mut game::State) {
    let mut buffer = [0; 2];
    io::stdin()
        .read_exact(&mut buffer)
        .expect("Failed to read key from STDIN");

    let (current_x, current_y) = state.cursor_pos;
    let (max_x, max_y) = get_size();

    if state.free_cursor {
        match buffer {
            [b'[', b'A'] => state.cursor_pos = (current_x, cmp::max(current_y - 1, 1)),
            [b'[', b'B'] => state.cursor_pos = (current_x, cmp::min(current_y + 1, max_y)),
            [b'[', b'C'] => state.cursor_pos = (cmp::min(current_x + 1, max_x), current_y),
            [b'[', b'D'] => state.cursor_pos = (cmp::max(current_x - 1, 1), current_y),
            _ => (),
        }
    } else if state.symbol_slots.contains(&state.cursor_pos) {
        match buffer {
            [b'[', b'A'] => state.cursor_pos = (current_x, cmp::max(current_y - 2, 2)),
            [b'[', b'B'] => state.cursor_pos = (current_x, cmp::min(current_y + 2, 6)),
            [b'[', b'C'] => state.cursor_pos = (cmp::min(current_x + 4, 11), current_y),
            [b'[', b'D'] => {
                state.cursor_pos = (cmp::max(current_x.saturating_sub(4), 3), current_y)
            }
            _ => (),
        }
    }
}

pub fn read_input(state: &mut game::State) -> anyhow::Result<()> {
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer)?;

    match buffer[0] {
        b'q' => restore_and_exit(),
        b's' => println!("{}", Ansi::ShowCursor),
        b'h' => println!("{}", Ansi::HideCursor),
        b'f' => state.free_cursor = !state.free_cursor,
        b'x' => game::attempt_placing(state, 'X'),
        b'o' => game::attempt_placing(state, 'O'),
        b' ' => game::attempt_placing(state, char::from(&state.current_player)),
        b'\x1B' => move_cursor(state),
        _ => (),
    }

    Ok(())
}

pub fn print_debug<T: fmt::Debug>(data: T) {
    print!("{}", Ansi::MoveCursor(1, get_size().1 - 2));
    println!("{:?}", data);
    io::stdout().flush().unwrap();
}
