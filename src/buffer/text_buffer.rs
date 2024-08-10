use std::{fs::File, io::BufReader, sync::Arc};
use ropey::Rope;
use crate::lsp::{client::{LspClient, ResponseReceiver, path_to_uri}, method::{completion::{CompletionFetch, CompletionParam}, didchange::DidChangeNotifyBuilder, hover::{HoverFetch, HoverParam}}};

use super::{ Buffer, CursorPos };

pub struct TextBuffer {
    filename: String,
    rope: Rope,
    lsp_client: Option<Arc<LspClient>>,
    version: i32,
}

impl TextBuffer {
    pub fn open(filename: &str) -> anyhow::Result<Self> {
        Ok(
            Self {
                filename: filename.to_owned(),
                rope: Rope::from_reader(BufReader::new(File::open(filename)?))?,
                lsp_client: None,
                version: 0,
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
                version: 0,
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
    async fn insert_char(&mut self, mut cursor: CursorPos, c: char) -> anyhow::Result<CursorPos> {

        if let Some(client) = &self.lsp_client {
            self.version += 1;
            DidChangeNotifyBuilder::new(&self.filename, self.version)?
                .insert(cursor, c.to_string())
                .notify(&client).await?;
        }

        let idx = self.rope.line_to_char(cursor.0);
        self.rope.insert(idx + cursor.1, &c.to_string());
        cursor.1 += 1;

        Ok(cursor)
    }
    async fn newline(&mut self, mut cursor: CursorPos) -> anyhow::Result<CursorPos> {
        // TODO: インデントがここに入るかもしれない
        // どう実装すればいい？
        
        if let Some(client) = &self.lsp_client {
            self.version += 1;
            DidChangeNotifyBuilder::new(&self.filename, self.version)?
                .insert(cursor, '\n'.to_string())
                .notify(&client).await?;
        }

        let idx = self.rope.line_to_char(cursor.0);
        self.rope.insert(idx + cursor.1, "\n");
        cursor.0 += 1;
        cursor.1 = 0;
        Ok(cursor)
    }
    async fn backspace(&mut self, mut cursor: CursorPos) -> anyhow::Result<CursorPos> {
        if cursor.1 == 0 {
            if cursor.0 > 0 {
                let end = cursor.clone();
                cursor.1 = self.rope.line(cursor.0 - 1).len_chars() - 1;
                let idx = self.rope.line_to_char(cursor.0);
                self.rope.remove(idx - 1..idx);
                cursor.0 -= 1;

                if let Some(client) = &self.lsp_client {
                    self.version += 1;
                    DidChangeNotifyBuilder::new(&self.filename, self.version)?
                        .delete(cursor, end)
                        .notify(&client).await?;
                }
            }
        }
        else {
            let idx = self.rope.line_to_char(cursor.0);
            self.rope.remove(idx + cursor.1 - 1..idx + cursor.1);
            cursor.1 -= 1;


            if let Some(client) = &self.lsp_client {
                self.version += 1;
                DidChangeNotifyBuilder::new(&self.filename, self.version)?
                    .delete(cursor, (cursor.0, cursor.1 + 1))
                    .notify(&client).await?;
            }
        }
        Ok(cursor)
    }

    async fn edit(&mut self, mut start: CursorPos, end: CursorPos, text: &str) -> anyhow::Result<CursorPos> {
        let sdx = self.rope.line_to_char(start.0);
        let edx = self.rope.line_to_char(end.0);
        self.rope.remove(sdx + start.1 .. edx + end.1);
        self.rope.insert(sdx + start.1, text);
        if let Some(client) = self.lsp_client.as_ref() {
            self.version += 1;
            DidChangeNotifyBuilder::new(&self.filename, self.version)?
                .edit(start, end, text.to_owned())
                .notify(client).await?;
        }
        for c in text.chars() {
            if c == '\n' {
                start.0 += 1;
                start.1 = 0;
            }
            else {
                start.1 += 1;
            }
        }
        Ok(start)
    }

    async fn hover(&self, cursor: CursorPos) -> anyhow::Result<Option<HoverFetch>> {
        match self.lsp_client {
            Some(ref lsp_client) => {
                let param = HoverParam::new(&self.filename, cursor)?;
                Ok(Some(HoverFetch::new(lsp_client, param).await?))
            }
            None => {
                Ok(None)
            }
        }
    }

    async fn completion(&self, cursor: CursorPos) -> anyhow::Result<Option<CompletionFetch>> {
        match self.lsp_client {
            Some(ref lsp_client) => {
                let param = CompletionParam::new(&self.filename, cursor)?;
                Ok(Some(CompletionFetch::new(lsp_client, param).await?))
            }
            None => {
                Ok(None)
            }
        }
    }
}
