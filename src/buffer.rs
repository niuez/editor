pub mod text_buffer;

use ropey::Rope;

use crate::lsp::{client::ResponseReceiver, method::hover::HoverFetch};

pub type CursorPos = (usize, usize);

pub trait Buffer {
    fn rope_clone(&self) -> Rope;
    fn len_lines(&self) -> usize;
    fn len_line_chars(&self, i: usize) -> usize;
    fn insert_char(&mut self, cursor: CursorPos, c: char) -> impl std::future::Future<Output=anyhow::Result<CursorPos>>;
    fn newline(&mut self, cursor: CursorPos) -> impl std::future::Future<Output=anyhow::Result<CursorPos>>;
    fn backspace(&mut self, cursor: CursorPos) -> impl std::future::Future<Output=anyhow::Result<CursorPos>>;
    fn hover(&self, cursor: CursorPos) -> impl std::future::Future<Output = anyhow::Result<Option<HoverFetch>>>;
}
