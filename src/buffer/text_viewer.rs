use std::{fs::File, io::BufReader};

use crate::terminal::Terminal;
use super::{ Draw, Input, BufferRect };
use ropey::Rope;

pub struct TextViewer {
    _filename: String,
    rope: Rope,
    top: usize,
    left: usize,
    cursor: (usize, usize),
}

impl TextViewer {
    pub fn open(filename: &str) -> anyhow::Result<Self> {
        Ok(
            TextViewer {
                _filename: filename.to_owned(),
                rope: Rope::from_reader(BufReader::new(File::open(filename)?))?,
                top: 0,
                left: 0,
                cursor: (0, 0),
            }
        )
    }

    fn fix_top_left(&mut self, rect: &BufferRect) {
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

impl Draw for TextViewer {
    fn draw_all(&mut self, rect: &BufferRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        self.fix_top_left(rect);
        for i in self.top..self.top + rect.h {
            if let Some(slice) = self.rope.get_line(i) {
                let len = slice.len_chars();
                terminal.set_cursor(rect.i + i - self.top, rect.j)?;
                if self.left <= len - 1 {
                    terminal.write(format!("{}", slice.slice(self.left..(len - 1).min(self.left + rect.w))).as_bytes())?;
                }
            }
        }
        Ok(())
    }
    fn draw_cursor(&mut self, rect: &BufferRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        self.fix_top_left(rect);
        assert!(self.top <= self.cursor.0);
        assert!(self.cursor.0 < self.top + rect.h);
        assert!(self.left <= self.cursor.1);
        assert!(self.cursor.1 < self.left + rect.w);

        terminal.set_cursor(self.cursor.0 - self.top + rect.i, self.cursor.1 - self.left + rect.j)?;
        Ok(())
    }
}

impl Input for TextViewer {
    fn move_left(&mut self) -> anyhow::Result<()> {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
        Ok(())
    }
    fn move_right(&mut self) -> anyhow::Result<()> {
        if self.cursor.1 + 1 < self.rope.line(self.cursor.0).len_chars() {
            self.cursor.1 += 1;
        }
        Ok(())
    }
    fn move_up(&mut self) -> anyhow::Result<()> {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
        self.cursor.1 = self.cursor.1.min(self.rope.line(self.cursor.0).len_chars() - 1);
        Ok(())
    }
    fn move_down(&mut self) -> anyhow::Result<()> {
        if self.cursor.0 + 1 < self.rope.len_lines() - 1 {
            self.cursor.0 += 1;
        }
        self.cursor.1 = self.cursor.1.min(self.rope.line(self.cursor.0).len_chars() - 1);
        Ok(())
    }
    fn insert_char(&mut self, c: char) -> anyhow::Result<()> {
        let idx = self.rope.line_to_char(self.cursor.0);
        self.rope.insert(idx + self.cursor.1, &c.to_string());
        self.cursor.1 += 1;
        Ok(())
    }
    fn newline(&mut self) -> anyhow::Result<()> {
        let idx = self.rope.line_to_char(self.cursor.0);
        self.rope.insert(idx + self.cursor.1, "\n");
        self.cursor.0 += 1;
        self.cursor.1 = 0;
        Ok(())
    }
    fn backspace(&mut self) -> anyhow::Result<()> {
        if self.cursor.1 == 0 {
            if self.cursor.0 > 0 {
                self.cursor.1 = self.rope.line(self.cursor.0 - 1).len_chars() - 1;
                let idx = self.rope.line_to_char(self.cursor.0);
                self.rope.remove(idx - 1..idx);
                self.cursor.0 -= 1;
            }
        }
        else {
            let idx = self.rope.line_to_char(self.cursor.0);
            self.rope.remove(idx + self.cursor.1 - 1..idx + self.cursor.1);
            self.cursor.1 -= 1;
        }
        Ok(())
    }
}
