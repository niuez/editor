use lsp_types::{CompletionParams, PartialResultParams, Position, Uri, WorkDoneProgressParams, request::{Request, Completion}};

use crate::{buffer::CursorPos, lsp::client::path_to_uri, viewer::completion_viewer::CompletionViewer};

use super::{LspFetch, LspParam, LspResult};

pub struct CompletionParam {
    uri: Uri,
    cursor: CursorPos,
}

impl CompletionParam {
    pub fn new<S: AsRef<std::path::Path>>(filename: S, cursor: CursorPos) -> anyhow::Result<Self> {
        Ok(Self {
            uri: path_to_uri(filename)?,
            cursor,
        })
    }
}

impl LspParam for CompletionParam {
    type ActualParam = CompletionParams;
    fn into_param(self) -> Self::ActualParam {
        CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: self.uri }, 
                position: lsp_types::Position { line: self.cursor.0 as u32, character: self.cursor.1 as u32 },
            },
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            partial_result_params: PartialResultParams { partial_result_token: None },
            context: None,
        }
    }
}


use lsp_types::CompletionResponse;

impl LspResult for Option<CompletionViewer> {
    type Response = Option<CompletionResponse>;
    type Param = CompletionParams;

    fn from_response(resp: Self::Response, param: Self::Param) -> Self {
        resp.map(|resp| CompletionViewer::new(resp, (param.text_document_position.position.line as usize, param.text_document_position.position.character as usize)))
    }
}

pub type CompletionFetch = LspFetch<Completion, Option<CompletionViewer>>;
