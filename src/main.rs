pub mod rawmode;
pub mod key;
pub mod terminal;
pub mod viewer;
pub mod buffer;
pub mod editor;
pub mod lsp;

use editor::Editor;


#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();
    //let mut editor = Editor::new()?;
    let mut editor = Editor::new_clangd().await?;
    editor.start().await?;
    Ok(())
}
