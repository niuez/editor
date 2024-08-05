pub mod rawmode;
pub mod key;
pub mod terminal;
pub mod viewer;
pub mod buffer;
pub mod editor;
pub mod lsp;

use editor::Editor;


#[tokio::main()]
async fn main() -> anyhow::Result<()> {
    //let mut editor = Editor::new()?;
    let mut editor = Editor::multi_viewer_test()?;
    editor.start()?;
    Ok(())
}
