use lsp_types::{MarkedString, request::HoverRequest};
use lsp_types::{Hover, HoverContents, HoverParams};

use crate::{buffer::CursorPos, lsp::client::{LspClient, TryGetResponse, ResponseReceiver, path_to_uri}};

use super::TryResult;

pub enum HoverFetch {
    Yet(HoverReceiver),
    Got(Option<HoverResult>),
}

pub struct HoverReceiver {
    receiver: ResponseReceiver<HoverRequest>,
}

pub struct HoverResult {
    pub text: String,
    pub pos: CursorPos,
}

impl HoverResult {
    fn from_response(resp: Hover, param: HoverParams) -> Self {
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
    }
}

impl HoverFetch {
    pub async fn new<S: AsRef<std::path::Path>>(client: &LspClient, filename: S, cursor: CursorPos) -> anyhow::Result<Self> {
        let receiver = client.request::<lsp_types::request::HoverRequest>(
            lsp_types::HoverParams {
                text_document_position_params: lsp_types::TextDocumentPositionParams {
                    text_document: lsp_types::TextDocumentIdentifier { uri: path_to_uri(&filename)? }, 
                    position: lsp_types::Position { line: cursor.0 as u32, character: cursor.1 as u32 },
                },
                work_done_progress_params: lsp_types::WorkDoneProgressParams { work_done_token: None }
            }
            ).await?;
        Ok(Self::Yet(HoverReceiver {
            receiver,
        }))
    }

    pub fn abort(self) {
        if let Self::Yet(receiver) = self {
            receiver.receiver.abort_request();
        }
    }

    pub async fn await_result(self) -> anyhow::Result<Option<HoverResult>> {
        match self {
            Self::Yet(receiver) => {
                let (resp, param) = receiver.receiver.await_result().await?;
                Ok(resp?.map(|resp| HoverResult::from_response(resp, param)))
            }
            Self::Got(r) => Ok(r),
        }
    }

    pub fn try_get_result(&mut self) -> anyhow::Result<Option<&Option<HoverResult>>> {
        let mut v = std::mem::replace(self, HoverFetch::Got(None));
        v = match v {
            Self::Yet(receiver) => {
                match receiver.receiver.try_get_response() {
                    TryGetResponse::Yet(receiver) => Self::Yet(HoverReceiver { receiver }),
                    TryGetResponse::Receive(resp) => Self::Got(resp.0?.map(|r| HoverResult::from_response(r, resp.1))),
                }
            }
            Self::Got(r) => Self::Got(r),
        };
        *self = v;
        Ok(match self {
            Self::Yet(_) => None,
            Self::Got(ref r) => Some(r),
        })
    }
}
