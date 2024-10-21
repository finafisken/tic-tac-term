use core::fmt;
use std::io::Write;
use libc::{c_int, ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

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

pub fn get_size() -> (u16, u16) {
	let mut size: winsize = unsafe { std::mem::zeroed() };
	unsafe {
		ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size);
	}

	(size.ws_col, size.ws_row)
}

pub extern "C" fn handle_signal(_signal: c_int) {
	// Clean up and restore cursor
	print!("{}", Ansi::ClearScreen);
	print!("{}", Ansi::ShowCursor);
	print!("{}", Ansi::MoveCursor(1, 1));
	std::io::stdout().flush().unwrap();
	std::process::exit(0)
}