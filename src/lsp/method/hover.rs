use lsp_types::{MarkedString, request::HoverRequest};
use lsp_types::{Hover, HoverContents, HoverParams, Uri};

use crate::{buffer::CursorPos, lsp::client::{LspClient, TryGetResponse, ResponseReceiver, path_to_uri}};

use super::{LspFetch, LspParam, LspResult};

pub struct HoverResult {
    pub text: String,
    pub pos: CursorPos,
}

impl LspResult for Option<HoverResult> {
    type Response = Option<Hover>;
    type Param = HoverParams;
    fn from_response(resp: Option<Hover>, param: HoverParams) -> Self {
        resp.map(|resp| {
            let pos = (param.text_document_position_params.position.line as usize, param.text_document_position_params.position.character as usize);

            match resp.contents {
                HoverContents::Markup(content) => {
                    HoverResult { text: content.value, pos }
                }
                HoverContents::Array(vec) => {
                    HoverResult {
                        text: vec.into_iter().map(|hover| {
                            match hover {
                                MarkedString::String(s) => s,
                                MarkedString::LanguageString(ls) => ls.value,
                            }
                        }).collect::<Vec<_>>().join("\n"),
                        pos,
                    }
                }
                HoverContents::Scalar(s) => {
                    HoverResult {
                        text: 
                            match s {
                                MarkedString::String(s) => s,
                                MarkedString::LanguageString(ls) => ls.value,
                            },
                            pos,
                    }
                }
            }
        })
    }
}

pub struct HoverParam {
    uri: Uri,
    cursor: CursorPos,
}

impl HoverParam {
    pub fn new<S: AsRef<std::path::Path>>(filename: S, cursor: CursorPos) -> anyhow::Result<Self> {
        Ok(Self {
            uri: path_to_uri(filename)?,
            cursor,
        })
    }
}

impl LspParam for HoverParam {
    type ActualParam = HoverParams;
    fn into_param(self) -> Self::ActualParam {
        lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: self.uri }, 
                position: lsp_types::Position { line: self.cursor.0 as u32, character: self.cursor.1 as u32 },
            },
            work_done_progress_params: lsp_types::WorkDoneProgressParams { work_done_token: None }
        }
    }
}

pub type HoverFetch = LspFetch<HoverRequest, Option<HoverResult>>;
