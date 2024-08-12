pub mod text_viewer;
pub mod hover_viewer;
pub mod completion_viewer;

use crate::{lsp::client::ResponseReceiver, terminal::Terminal};

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
    fn insert_char(&mut self, _: char) -> impl std::future::Future<Output=anyhow::Result<()>> { async { Ok(()) } }
    fn newline(&mut self) -> impl std::future::Future<Output=anyhow::Result<()>> { async { Ok(()) } }
    fn backspace(&mut self) -> impl std::future::Future<Output=anyhow::Result<()>> { async { Ok(()) } }
    fn hover(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>>;
    fn completion(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>>;
    fn do_completion(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>>;
    fn completion_next(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>>;
    fn completion_prev(&mut self) -> impl std::future::Future<Output = anyhow::Result<()>>;
}

pub trait Viewer: Draw + Input {}

