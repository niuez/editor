use crate::viewer::{ Draw, Input, ViewerRect, text_viewer::TextViewer };
use crate::rawmode::RawMode;
use crate::terminal::Terminal;
use anyhow::{ anyhow, Context };
use crate::key::Key;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    _mode: RawMode,
    stdin: std::io::Stdin,
    terminal: Terminal,

    insert_char_buffer: Vec<u8>,
    mode: Mode,

    text_viewer: TextViewer,
    rect: ViewerRect,
}

impl Editor {
    pub fn new() -> anyhow::Result<Editor> {
        let terminal = Terminal::new()?;
        let rect = ViewerRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 };
        Ok(Editor {
            _mode: RawMode::enable_raw_mode().context("enable raw mode failed")?,
            stdin: std::io::stdin(),
            terminal,
            insert_char_buffer: vec![],
            mode: Mode::Normal,
            text_viewer: TextViewer::open("./test.txt")?,
            rect,
        })
    }

    fn update_all(&mut self) -> anyhow::Result<()> {
        self.terminal.clear_all()?;
        self.text_viewer.draw_all(&self.rect, &mut self.terminal)?;
        self.text_viewer.draw_cursor(&self.rect, &mut self.terminal)?;
        self.terminal.flush()
    }

    pub fn normal_input(&mut self, key: Key) -> anyhow::Result<()> {
             if key == Key::char(b'j') { self.text_viewer.move_down() }
        else if key == Key::char(b'k') { self.text_viewer.move_up() }
        else if key == Key::char(b'h') { self.text_viewer.move_left() }
        else if key == Key::char(b'l') { self.text_viewer.move_right() }
        else if key == Key::char(b'i') { self.mode = Mode::Insert; Ok(()) }
        else { Ok(()) }
    }

    pub fn insert_input(&mut self, key: Key) -> anyhow::Result<()> {
        if self.insert_char_buffer.len() > 0 {
            if let Key::Character(ch) = key {
                self.insert_char_buffer.push(ch);
            }
            else {
                return Err(anyhow!("bugged char?"))
            }
        }
        else if key == Key::escape() {
            self.mode = Mode::Normal;
        }
        else if key == Key::backspace() {
            self.text_viewer.backspace()?;
        }
        else if key == Key::char(b'\r') {
            self.text_viewer.newline()?;
        }
        else if let Key::Character(ch) = key {
            if ch >= 32 {
                self.insert_char_buffer.push(ch);
            }
        }
        if self.insert_char_buffer.len() > 0 {
            if let Ok(st) = String::from_utf8(self.insert_char_buffer.clone()) {
                for c in st.chars() {
                    self.text_viewer.insert_char(c)?;
                }
                self.insert_char_buffer.clear();
            }
        }
        Ok(())
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.update_all()?;
        loop {
            if let Some(key) = Key::try_read_from_stdin(&mut self.stdin)? {
                if key == Key::ctrl(b'c') {
                    break;
                }
                else if self.mode == Mode::Normal {
                    self.normal_input(key)?;
                }
                else if self.mode == Mode::Insert {
                    self.insert_input(key)?;
                }
                self.update_all()?;
            }
        }
        Ok(())
    }
}
