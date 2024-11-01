pub struct State {
    pub board: [char; 9],
    pub board_pos: (u16, u16),
    pub cursor_pos: (u16, u16)
}

pub fn new() -> State {
	State { board: [' '; 9], board_pos: (1,1), cursor_pos: (2,2)}
}