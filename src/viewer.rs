pub mod text_viewer;

use crate::terminal::Terminal;

#[derive(Debug, Clone)]
pub struct ViewerRect {
    pub h: usize,
    pub w: usize,
    pub i: usize,
    pub j: usize,
}

pub trait Draw {
    fn draw_all(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()>;
    fn draw_cursor(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()>;
}

pub trait Input {
    fn move_left(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn move_right(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn move_up(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn move_down(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn insert_char(&mut self, _: char) -> anyhow::Result<()> { Ok(()) }
    fn newline(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn backspace(&mut self) -> anyhow::Result<()> { Ok(()) }
}

pub trait Buffer: Draw + Input {}

