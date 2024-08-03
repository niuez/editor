use std::{cell::RefCell, fs::File, io::BufReader, rc::Rc};

use crate::{buffer::Buffer, terminal::Terminal};
use super::{Draw, Input, Viewer, ViewerRect};

pub struct TextViewer<B: Buffer> {
    buffer: Rc<RefCell<B>>,
    top: usize,
    left: usize,
    cursor: (usize, usize),
}

impl<B: Buffer> TextViewer<B> {
    pub fn open(buffer: Rc<RefCell<B>>) -> anyhow::Result<Self> {
        Ok(
            TextViewer {
                buffer,
                top: 0,
                left: 0,
                cursor: (0, 0),
            }
        )
    }

    fn fix_top_left(&mut self, rect: &ViewerRect) {
        if self.top > self.cursor.0 {
            self.top = self.cursor.0;
        }
        if self.cursor.0 >= self.top + rect.h {
            self.top = self.cursor.0 - rect.h + 1;
        }

        if self.left > self.cursor.1 {
            self.left = self.cursor.1;
        }
        if self.cursor.1 >= self.left + rect.w {
            self.left = self.cursor.1 - rect.w + 1;
        }
    }
}

impl<B: Buffer> Draw for TextViewer<B> {
    fn draw_all(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        self.fix_top_left(rect);
        let rope = self.buffer.borrow().rope_clone();
        for i in self.top..self.top + rect.h {
            if let Some(slice) = rope.get_line(i) {
                let len = slice.len_chars();
                terminal.set_cursor(rect.i + i - self.top, rect.j)?;
                if self.left <= len - 1 {
                    terminal.write(format!("{}", slice.slice(self.left..(len - 1).min(self.left + rect.w))).as_bytes())?;
                }
            }
        }
        Ok(())
    }
    fn draw_cursor(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        self.fix_top_left(rect);
        assert!(self.top <= self.cursor.0);
        assert!(self.cursor.0 < self.top + rect.h);
        assert!(self.left <= self.cursor.1);
        assert!(self.cursor.1 < self.left + rect.w);

        terminal.set_cursor(self.cursor.0 - self.top + rect.i, self.cursor.1 - self.left + rect.j)?;
        Ok(())
    }
}

impl<B: Buffer> Input for TextViewer<B> {
    fn move_left(&mut self) -> anyhow::Result<()> {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
        Ok(())
    }
    fn move_right(&mut self) -> anyhow::Result<()> {
        if self.cursor.1 + 1 < self.buffer.borrow().len_line_chars(self.cursor.0) {
            self.cursor.1 += 1;
        }
        Ok(())
    }
    fn move_up(&mut self) -> anyhow::Result<()> {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
        self.cursor.1 = self.cursor.1.min(self.buffer.borrow().len_line_chars(self.cursor.0) - 1);
        Ok(())
    }
    fn move_down(&mut self) -> anyhow::Result<()> {
        if self.cursor.0 + 1 < self.buffer.borrow().len_lines() - 1 {
            self.cursor.0 += 1;
        }
        self.cursor.1 = self.cursor.1.min(self.buffer.borrow().len_line_chars(self.cursor.0) - 1);
        Ok(())
    }
    fn insert_char(&mut self, c: char) -> anyhow::Result<()> {
        self.cursor = self.buffer.borrow_mut().insert_char(self.cursor, c)?;
        Ok(())
    }
    fn newline(&mut self) -> anyhow::Result<()> {
        self.cursor = self.buffer.borrow_mut().newline(self.cursor)?;
        Ok(())
    }
    fn backspace(&mut self) -> anyhow::Result<()> {
        self.cursor = self.buffer.borrow_mut().newline(self.cursor)?;
        Ok(())
    }
}

impl<B: Buffer> Viewer for TextViewer<B> {}
