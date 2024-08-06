use std::{fs::File, io::BufReader, sync::Arc};
use ropey::Rope;
use crate::lsp::{client::{LspClient, ResponseReceiver, path_to_uri}, method::hover::HoverFetch};

use super::{ Buffer, CursorPos };

pub struct TextBuffer {
    filename: String,
    rope: Rope,
    lsp_client: Option<Arc<LspClient>>,
}

impl TextBuffer {
    pub fn open(filename: &str) -> anyhow::Result<Self> {
        Ok(
            Self {
                filename: filename.to_owned(),
                rope: Rope::from_reader(BufReader::new(File::open(filename)?))?,
                lsp_client: None,
            }
        )
    }

    pub async fn open_with_lsp(filename: &str, lsp_client: Arc<LspClient>) -> anyhow::Result<Self> {
        let text = String::from_utf8_lossy(&std::fs::read(filename)?).to_string();

        // TODO: cpp to actual language id
        lsp_client.notify::<lsp_types::notification::DidOpenTextDocument>(
            lsp_types::DidOpenTextDocumentParams {
                text_document: lsp_types::TextDocumentItem { uri: path_to_uri(filename)?, language_id: "cpp".to_owned(), version: 0, text: text.clone() }
            }).await?;

        Ok(
            Self {
                filename: filename.to_owned(),
                rope: Rope::from_reader(BufReader::new(File::open(filename)?))?,
                lsp_client: Some(lsp_client),
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
    async fn hover(&self, cursor: CursorPos) -> anyhow::Result<Option<HoverFetch>> {
        match self.lsp_client {
            Some(ref lsp_client) => {
                Ok(Some(HoverFetch::new(lsp_client, &self.filename, cursor).await?))
            }
            None => {
                Ok(None)
            }
        }
    }
}
