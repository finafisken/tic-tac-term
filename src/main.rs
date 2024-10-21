use std::{io::{self, Write}, thread, time};
use libc::{signal, SIGINT, SIGTERM};
use terminal::Ansi;

mod terminal;


fn main() -> anyhow::Result<()> {
    unsafe {
        signal(SIGINT, terminal::handle_signal as usize);
        signal(SIGTERM, terminal::handle_signal as usize);
    }

    loop {
        print!("{}", Ansi::HideCursor);
        print!("{}", Ansi::ClearScreen);
        let (cols, rows) = terminal::get_size();
        print!("{}", Ansi::MoveCursor(cols/2, rows/2));
        print!("{} columns, {} rows", cols, rows);
        io::stdout().flush()?;
        thread::sleep(time::Duration::from_secs(1));
    }

    Ok(())
}
