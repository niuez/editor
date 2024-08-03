pub mod rawmode;
pub mod key;
pub mod terminal;
pub mod buffer;
pub mod editor;

use editor::Editor;


fn main() -> anyhow::Result<()> {
    let mut editor = Editor::new()?;
    editor.start()?;
    Ok(())
}
