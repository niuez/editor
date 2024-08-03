pub mod rawmode;
pub mod key;
pub mod terminal;
pub mod buffer;
pub mod editor;

use anyhow::Context;
use buffer::text_viewer::TextViewer;
use buffer::{BufferRect, Draw, Input};


fn main() -> anyhow::Result<()> {
    let _mode = rawmode::RawMode::enable_raw_mode().context("enable raw mode failed");

    let mut stdin = std::io::stdin();
    let mut viewer = TextViewer::open("./src/buffer/text_viewer.rs")?;
    let mut terminal = terminal::Terminal::new()?;
    terminal.clear_all()?;
    viewer.draw_all(&BufferRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 }, &mut terminal)?;
    viewer.draw_cursor(&BufferRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 }, &mut terminal)?;
    terminal.flush()?;
    loop {
        if let Some(key) = key::Key::try_read_from_stdin(&mut stdin)? {
            viewer.input(key)?;
            terminal.clear_all()?;
            viewer.draw_all(&BufferRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 }, &mut terminal)?;
            viewer.draw_cursor(&BufferRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 }, &mut terminal)?;
            terminal.flush()?;
            if key == key::Key::ctrl(b'c') {
                break;
            }
        }
    }
    Ok(())
}
