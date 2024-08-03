use std::io::{Stdout, Write};

#[derive(Clone, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Color {
    fn default() -> Self {
        Color { r: 0, g: 0, b: 0, }
    }
}

pub struct Terminal {
    stdout: Stdout,
    h: usize,
    w: usize,
}

impl Terminal {
    pub fn new() -> anyhow::Result<Self> {
        let stdout = std::io::stdout();
        let (h, w) = get_window_size()?;
        Ok(Self {
            stdout,
            h: h as usize,
            w: w as usize,
        })
    }

    pub fn height(&self) -> usize { self.h }
    pub fn width(&self) -> usize { self.w }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        Ok(self.stdout.flush()?)
    }

    pub fn clear_all(&mut self) -> anyhow::Result<()> {
        self.write(b"\x1b[2J")
    }

    pub fn clear_cursor_line(&mut self) -> anyhow::Result<()> {
        self.write(b"\x1b[2K")
    }

    pub fn set_cursor(&mut self, i: usize, j: usize) -> anyhow::Result<()> {
        self.write(format!("\x1b[{};{}H", i + 1, j + 1).as_bytes())
    }

    pub fn set_fg(&mut self, fg: Color) -> anyhow::Result<()> {
        self.write(format!("\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b).as_bytes())
    }

    pub fn set_bg(&mut self, bg: Color) -> anyhow::Result<()> {
        self.write(format!("\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b).as_bytes())
    }

    pub fn write(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        Ok(self.stdout.write_all(buf)?)
    }
}

use libc::{ winsize, STDOUT_FILENO, TIOCGWINSZ };
use anyhow::anyhow;

fn get_window_size() -> anyhow::Result<(u16, u16)> {
    let ws = winsize {
        ws_col: 0,
        ws_row: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe {
        if libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == -1 || ws.ws_col == 0 {
            return Err(anyhow!{ "get_window_size: ioctl failed" })
        }
    }
    Ok((ws.ws_row, ws.ws_col))
}
