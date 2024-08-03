use std::{fs::File, io::BufReader};

use anyhow::anyhow;
use crate::{key::Key, terminal::Terminal};
use super::{ Draw, Input, BufferRect };
use ropey::Rope;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Mode {
    Normal,
    Insert,
}

pub struct TextViewer {
    filename: String,
    rope: Rope,
    top: usize,
    left: usize,
    cursor: (usize, usize),

    char_buffer: Vec<u8>,
    mode: Mode,
}

impl TextViewer {
    pub fn open(filename: &str) -> anyhow::Result<Self> {
        Ok(
            TextViewer {
                filename: filename.to_owned(),
                rope: Rope::from_reader(BufReader::new(File::open(filename)?))?,
                top: 0,
                left: 0,
                cursor: (0, 0),

                char_buffer: Vec::new(),
                mode: Mode::Normal,
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
    fn input(&mut self, key: Key) -> anyhow::Result<()> {
        match self.mode {
            Mode::Normal => self.normal_input(key),
            Mode::Insert => self.insert_input(key),
        }
    }
}

impl TextViewer {
    fn normal_input(&mut self, key: Key) -> anyhow::Result<()> {
        match key {
            Key::Character(b'j') => {
                if self.cursor.0 + 1 < self.rope.len_lines() - 1 {
                    self.cursor.0 += 1;
                }
                self.cursor.1 = self.cursor.1.min(self.rope.line(self.cursor.0).len_chars() - 1);
            }
            Key::Character(b'k') => {
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                }
                self.cursor.1 = self.cursor.1.min(self.rope.line(self.cursor.0).len_chars() - 1);
            }
            Key::Character(b'l') => {
                if self.cursor.1 + 1 < self.rope.line(self.cursor.0).len_chars() {
                    self.cursor.1 += 1;
                }
            }
            Key::Character(b'h') => {
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
            }
            Key::Character(b'i') => {
                self.mode = Mode::Insert;
            }
            _ => {}
        }
        Ok(())
    }

    fn newline(&mut self) {
        let idx = self.rope.line_to_char(self.cursor.0);
        self.rope.insert(idx + self.cursor.1, "\n");
        self.cursor.0 += 1;
        self.cursor.1 = 0;
    }

    fn backspace(&mut self) {
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
    }

    fn insert(&mut self, s: String) {
        let idx = self.rope.line_to_char(self.cursor.0);
        self.rope.insert(idx + self.cursor.1, &s);
        self.cursor.1 += s.chars().count();
    }

    fn insert_input(&mut self, key: Key) -> anyhow::Result<()> {
        eprintln!("{:?}", key);
        if self.char_buffer.len() > 0 {
            if let Key::Character(ch) = key {
                self.char_buffer.push(ch);
            }
            else {
                return Err(anyhow!("bugged char?"))
            }
        }
        else if key == Key::escape() {
            self.mode = Mode::Normal;
        }
        else if key == Key::backspace() {
            self.backspace();
        }
        else if key == Key::char(b'\r') {
            self.newline();
        }
        else if let Key::Character(ch) = key {
            if ch >= 32 {
                self.char_buffer.push(ch);
            }
        }
        if self.char_buffer.len() > 0 {
            if let Ok(st) = String::from_utf8(self.char_buffer.clone()) {
                self.insert(st);
                self.char_buffer.clear();
            }
        }
        Ok(())
    }
}
