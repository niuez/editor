pub mod text_viewer;

use crate::{key::Key, terminal::Terminal};

#[derive(Debug, Clone)]
pub struct BufferRect {
    pub h: usize,
    pub w: usize,
    pub i: usize,
    pub j: usize,
}

pub trait Draw {
    fn draw_all(&mut self, rect: &BufferRect, terminal: &mut Terminal) -> anyhow::Result<()>;
    fn draw_cursor(&mut self, rect: &BufferRect, terminal: &mut Terminal) -> anyhow::Result<()>;
}

pub trait Input {
    fn input(&mut self, key: Key) -> anyhow::Result<()>;
}

