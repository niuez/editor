mod msg;
mod stdio;
mod error;
mod socket;

use std::{str::FromStr, sync::{Arc, Mutex}};


use lsp_types::{*, request::Initialize, };
use msg::{ Message, Request, RequestId, Notification };

use std::collections::HashMap;

use crossbeam_channel::{bounded, Receiver, Sender};

fn main() {
    let mut command = std::process::Command::new("clangd")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::io::stderr())
        .spawn()
        .expect("failed to launch");

    let mut to_server = command.stdin.take().unwrap();
    let from_server = command.stdout.take().unwrap();
    let mut server_reader = std::io::BufReader::new(from_server);

    let (reader_sender, reader_receiver) = bounded::<Message>(0);
    let reader = std::thread::Builder::new()
        .name("LspServerReader".to_owned())
        .spawn(move || -> std::io::Result<()> {
            let mut server_reader = server_reader;
            while let Some(msg) = Message::read(&mut server_reader)? {
                reader_sender.send(msg).expect("receiver was dropped")
            }
            Ok(())
        })
        .unwrap();

    let client_capabilities = ClientCapabilities {
        text_document: Some(TextDocumentClientCapabilities { hover: Some(HoverClientCapabilities { dynamic_registration: Some(true), content_format: Some(vec![MarkupKind::PlainText, MarkupKind::Markdown]) }), ..Default::default() }),
        ..Default::default()
    };

    let work = WorkspaceFolder {
        uri: Uri::from_str(&format!("file://{}", std::path::absolute("./").unwrap().to_str().unwrap())).unwrap(),
        name: "test".to_owned(),
    };

    let init_params = InitializeParams {
        process_id: Some(std::process::id()),
        capabilities: client_capabilities,
        workspace_folders: Some(vec![work]),
        ..Default::default()
    };


    let mut it = 0;
    let init_req = Request::new(RequestId::from(it), "initialize".to_owned(), init_params);
    it += 1;
    let init_msg = Message::from(init_req);

    init_msg.write(&mut to_server).unwrap();

    for msg in reader_receiver {
        match msg {
            Message::Response(req) => {
                if req.id == RequestId::from(0) {
                    let init_notify = Message::from(Notification::new("initialized".to_owned(), InitializedParams {}));
                    init_notify.write(&mut to_server).unwrap();

                    let uri = Uri::from_str(&format!("file://{}", std::path::absolute("./1.cpp").unwrap().to_str().unwrap())).unwrap();
                    let init_notify = Message::from(Notification::new("textDocument/didOpen".to_owned(), DidOpenTextDocumentParams {
                        text_document: TextDocumentItem::new(
                                           uri.clone(), "rust".to_owned(), 0, 
                                           String::from_utf8_lossy(&std::fs::read("./1.cpp").unwrap()).to_string()
                                        )
                    }));
                    init_notify.write(&mut to_server).unwrap();

                    let uri = Uri::from_str(&format!("file://{}", std::path::absolute("./1.cpp").unwrap().to_str().unwrap())).unwrap();
                    let text_doc_id = TextDocumentIdentifier { uri, };
                    let pos = Position::new(0, 0);
                    let text_doc_pos = TextDocumentPositionParams::new(text_doc_id, pos);
                    let hover_req = Message::from(Request::new(RequestId::from(it), "textDocument/hover".to_owned(), HoverParams { text_document_position_params: text_doc_pos, work_done_progress_params: WorkDoneProgressParams::default() }));
                    hover_req.write(&mut to_server).unwrap();
                }
                if req.id == RequestId::from(1) {
                    match req.extract::<Hover>() {
                        Ok(hover) => {
                            eprintln!("hover success!: {:?}", hover);
                        }
                        Err(err) => {
                            eprintln!("err: {:?}", err);
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
