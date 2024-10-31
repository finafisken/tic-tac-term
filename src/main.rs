use std::{char, io::{self, Write}, thread, time};
use libc::{signal, SIGINT, SIGTERM};
use terminal::Ansi;

mod terminal;


fn draw_board(state: [char; 9]) {
    print!("{}", Ansi::MoveCursor(1,1));
    println!("┌───┬───┬───┐");
    println!("│ {} │ {} │ {} │", state[0], state[1], state[2]);
    println!("├───┼───┼───┤");
    println!("│ {} │ {} │ {} │", state[3], state[4], state[5]);
    println!("├───┼───┼───┤");
    println!("│ {} │ {} │ {} │", state[6], state[7], state[8]);
    println!("└───┴───┴───┘");

}

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode();
    print!("{}", Ansi::HideCursor);

    unsafe {
        signal(SIGINT, terminal::handle_signal as usize);
        signal(SIGTERM, terminal::handle_signal as usize);
    }

    loop {
        print!("{}", Ansi::ClearScreen);
        draw_board(['1', '2', '3', '4', '5','6', '7', '8', ' ']);
        io::stdout().flush()?;
        terminal::read_input()?;
        thread::sleep(time::Duration::from_millis(300));
    }

    Ok(())
}
