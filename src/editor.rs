use std::cell::RefCell;
use std::rc::Rc;

use crate::buffer::text_buffer::TextBuffer;
use crate::viewer::{ Draw, Input, Viewer, ViewerRect, text_viewer::TextViewer };
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

    buffers: Vec<Rc<RefCell<TextBuffer>>>,
    viewers: Vec<(Box<dyn Viewer>, ViewerRect)>,
    active: usize,
}

impl Editor {
    pub fn new() -> anyhow::Result<Editor> {
        let terminal = Terminal::new()?;
        let rect = ViewerRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 };
        let buffer = Rc::new(RefCell::new(TextBuffer::open("./test.txt")?));
        Ok(Editor {
            _mode: RawMode::enable_raw_mode().context("enable raw mode failed")?,
            stdin: std::io::stdin(),
            terminal,
            insert_char_buffer: vec![],
            mode: Mode::Normal,

            buffers: vec![buffer.clone()],
            viewers: vec![(Box::new(TextViewer::open(buffer.clone())?), rect)],
            active: 0,
        })
    }

    fn update_all(&mut self) -> anyhow::Result<()> {
        self.terminal.clear_all()?;
        for (viewer, rect) in self.viewers.iter_mut() {
            viewer.draw_all(rect, &mut self.terminal)?;
        }
        let active_rect = self.viewers[self.active].1.clone();
        self.viewers[self.active].0.draw_cursor(&active_rect, &mut self.terminal)?;
        self.terminal.flush()
    }

    fn normal_input(&mut self, key: Key) -> anyhow::Result<()> {
             if key == Key::char(b'j') { self.viewers[self.active].0.move_down() }
        else if key == Key::char(b'k') { self.viewers[self.active].0.move_up() }
        else if key == Key::char(b'h') { self.viewers[self.active].0.move_left() }
        else if key == Key::char(b'l') { self.viewers[self.active].0.move_right() }
        else if key == Key::char(b'i') { self.mode = Mode::Insert; Ok(()) }
        else if key == Key::ctrl(b'w') { self.active = (self.active + 1) % self.viewers.len(); Ok(()) }
        else { Ok(()) }
    }

    fn insert_input(&mut self, key: Key) -> anyhow::Result<()> {
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
            self.viewers[self.active].0.backspace()?;
        }
        else if key == Key::char(b'\r') {
            self.viewers[self.active].0.newline()?;
        }
        else if let Key::Character(ch) = key {
            if ch >= 32 {
                self.insert_char_buffer.push(ch);
            }
        }
        if self.insert_char_buffer.len() > 0 {
            if let Ok(st) = String::from_utf8(self.insert_char_buffer.clone()) {
                for c in st.chars() {
                    self.viewers[self.active].0.insert_char(c)?;
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

impl Editor {
    pub fn multi_viewer_test() -> anyhow::Result<Self> {
        let terminal = Terminal::new()?;
        let w = terminal.width() / 2;
        let rect1 = ViewerRect { h: terminal.height(), w, i: 0, j: 0 };
        let rect2 = ViewerRect { h: terminal.height(), w, i: 0, j: w };
        let buffer = Rc::new(RefCell::new(TextBuffer::open("./test.txt")?));
        Ok(Editor {
            _mode: RawMode::enable_raw_mode().context("enable raw mode failed")?,
            stdin: std::io::stdin(),
            terminal,
            insert_char_buffer: vec![],
            mode: Mode::Normal,

            buffers: vec![buffer.clone()],
            viewers: vec![
                (Box::new(TextViewer::open(buffer.clone())?), rect1),
                (Box::new(TextViewer::open(buffer.clone())?), rect2),
            ],
            active: 0,
        })
    }
}
