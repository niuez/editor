use std::{fs::File, io::BufReader};
use ropey::Rope;
use super::{ Buffer, CursorPos };

pub struct TextBuffer {
    _filename: String,
    rope: Rope,
}

impl TextBuffer {
    pub fn open(filename: &str) -> anyhow::Result<Self> {
        Ok(
            Self {
                _filename: filename.to_owned(),
                rope: Rope::from_reader(BufReader::new(File::open(filename)?))?,
            }
        )
    }
}

impl Buffer for TextBuffer {
    fn rope_clone(&self) -> Rope {
        self.rope.clone()
    }
    fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }
    fn len_line_chars(&self, i: usize) -> usize {
        self.rope.line(i).len_chars()
    }
    fn insert_char(&mut self, mut cursor: CursorPos, c: char) -> anyhow::Result<CursorPos> {
        let idx = self.rope.line_to_char(cursor.0);
        self.rope.insert(idx + cursor.1, &c.to_string());
        cursor.1 += 1;
        Ok(cursor)
    }
    fn newline(&mut self, mut cursor: CursorPos) -> anyhow::Result<CursorPos> {
        let idx = self.rope.line_to_char(cursor.0);
        self.rope.insert(idx + cursor.1, "\n");
        cursor.0 += 1;
        cursor.1 = 0;
        Ok(cursor)
    }
    fn backspace(&mut self, mut cursor: CursorPos) -> anyhow::Result<CursorPos> {
        if cursor.1 == 0 {
            if cursor.0 > 0 {
                cursor.1 = self.rope.line(cursor.0 - 1).len_chars() - 1;
                let idx = self.rope.line_to_char(cursor.0);
                self.rope.remove(idx - 1..idx);
                cursor.0 -= 1;
            }
        }
        else {
            let idx = self.rope.line_to_char(cursor.0);
            self.rope.remove(idx + cursor.1 - 1..idx + cursor.1);
            cursor.1 -= 1;
        }
        Ok(cursor)
    }
}
