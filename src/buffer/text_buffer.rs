use std::{fs::File, io::BufReader, sync::Arc};
use ropey::Rope;
use crate::lsp::client::{LspClient, ResponseReceiver, path_to_uri};

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
    async fn hover(&self, cursor: CursorPos) -> anyhow::Result<Option<ResponseReceiver<lsp_types::request::HoverRequest>>> {
        match self.lsp_client {
            Some(ref lsp_client) => {
                let receiver = lsp_client.request::<lsp_types::request::HoverRequest>(
                    lsp_types::HoverParams {
                        text_document_position_params: lsp_types::TextDocumentPositionParams {
                            text_document: lsp_types::TextDocumentIdentifier { uri: path_to_uri(&self.filename)? }, 
                            position: lsp_types::Position { line: cursor.0 as u32, character: cursor.1 as u32 },
                        },
                        work_done_progress_params: lsp_types::WorkDoneProgressParams { work_done_token: None }
                    }
                ).await?;
                Ok(Some(receiver))

            }
            None => {
                Ok(None)
            }
        }
    }
}
