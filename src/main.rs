pub mod rawmode;
pub mod key;
pub mod terminal;
pub mod viewer;
pub mod buffer;
pub mod editor;

use editor::Editor;


fn main() -> anyhow::Result<()> {
    //let mut editor = Editor::new()?;
    let mut editor = Editor::multi_viewer_test()?;
    editor.start()?;
    Ok(())
}
