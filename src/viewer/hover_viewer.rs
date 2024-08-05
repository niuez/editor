use ropey::Rope;

use crate::terminal::Terminal;

use super::{Draw, ViewerRect};


pub struct HoverViewer {
    s: String,
}

impl HoverViewer {
    pub fn new(s: String) -> Self {
        Self { s }
    }
    pub fn empty() -> Self {
        Self { s: String::new() }
    }
}

impl Draw for HoverViewer {
    fn draw_all(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        if self.s.is_empty() {
            return Ok(());
        }
        let rope = Rope::from_str(&self.s);
        for i in 0..rect.h {
            if let Some(slice) = rope.get_line(i) {
                let len = slice.len_chars();
                terminal.set_cursor(rect.i + i, rect.j)?;
                if 0 < len {
                    terminal.write(format!("{}", slice.slice(0..(len - 1).min(0 + rect.w))).as_bytes())?;
                }
            }
        }
        Ok(())
    }
    fn draw_cursor(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        Ok(())
    }
}
