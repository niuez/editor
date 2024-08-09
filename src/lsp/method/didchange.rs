use lsp_types::{DidChangeTextDocumentParams, Position, TextDocumentContentChangeEvent, VersionedTextDocumentIdentifier, notification::DidChangeTextDocument};

use crate::{buffer::CursorPos, lsp::client::{LspClient, path_to_uri}};

pub struct DidChangeNotifyBuilder {
    text_document: VersionedTextDocumentIdentifier,
    changes: Vec<TextDocumentContentChangeEvent>,
}

impl DidChangeNotifyBuilder {
    pub fn new<S: AsRef<std::path::Path>>(filename: S, version: i32) -> anyhow::Result<Self> {
        Ok(Self {
            text_document: VersionedTextDocumentIdentifier { uri: path_to_uri(filename)?, version, },
            changes: vec![],
        })
    }

    pub fn full(mut self, full_text: String) -> Self {
        self.changes.push(TextDocumentContentChangeEvent { range: None, range_length: None, text: full_text });
        self
    }

    pub fn insert(mut self, insert_pos: CursorPos, text: String) -> Self {
        self.changes.push(TextDocumentContentChangeEvent {
            range: Some(lsp_types::Range {
                start: Position::new(insert_pos.0 as u32, insert_pos.1 as u32),
                end: Position::new(insert_pos.0 as u32, insert_pos.1 as u32),
            }),
            range_length: None,
            text,
        });
        self
    }

    pub fn delete(mut self, delete_start: CursorPos, delete_end: CursorPos) -> Self {
        self.changes.push(TextDocumentContentChangeEvent {
            range: Some(lsp_types::Range {
                start: Position::new(delete_start.0 as u32, delete_start.1 as u32),
                end: Position::new(delete_end.0 as u32, delete_end.1 as u32),
            }),
            range_length: None,
            text: String::new(),
        });
        self
    }

    pub async fn notify(self, client: &LspClient) -> anyhow::Result<Self> {
        client.notify::<DidChangeTextDocument>(
            DidChangeTextDocumentParams {
                text_document: self.text_document.clone(),
                content_changes: self.changes.clone(),
            }).await?;
        Ok(self)
    }
}
