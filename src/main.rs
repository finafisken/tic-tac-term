use core::{fmt, time};
use std::{io::{self, Write}, thread};

enum ANSI {
    HideCursor,           // "\033[?25l"
    ShowCursor,           // "\033[?25h"
    ClearScreen,          //  "\033[2J"
    MoveCursor(u32, u32), // "\033[%d;%dH" %d num
}

impl fmt::Display for ANSI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ANSI::HideCursor => write!(f, "\x1B[?25l"),
            ANSI::ShowCursor => write!(f, "\x1B[?25h"),
            ANSI::ClearScreen => write!(f, "\x1B[2J"),
            ANSI::MoveCursor(x, y) => write!(f, "\x1B[{};{}H", x, y),
        }
    }
}

fn main() {
    // println!("Hello, world!");

    print!("{}", ANSI::HideCursor);
    print!("{}", ANSI::ClearScreen);
    print!("{}", ANSI::MoveCursor(10, 10));
    print!("yeet!");
    io::stdout().flush();
    thread::sleep(time::Duration::from_secs(3));

    print!("{}", ANSI::ShowCursor);
}
