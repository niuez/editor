pub mod text_buffer;

use ropey::Rope;

pub type CursorPos = (usize, usize);

pub trait Buffer {
    fn rope_clone(&self) -> Rope;
    fn len_lines(&self) -> usize;
    fn len_line_chars(&self, i: usize) -> usize;
    fn insert_char(&mut self, cursor: CursorPos, c: char) -> anyhow::Result<CursorPos>;
    fn newline(&mut self, cursor: CursorPos) -> anyhow::Result<CursorPos>;
    fn backspace(&mut self, cursor: CursorPos) -> anyhow::Result<CursorPos>;
}
