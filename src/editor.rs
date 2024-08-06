use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::buffer::text_buffer::TextBuffer;
use crate::lsp::client::{LspClient, LspClientStartArg, path_to_uri};
use crate::viewer::hover_viewer::HoverViewer;
use crate::viewer::{ Draw, Input, Viewer, ViewerRect, text_viewer::TextViewer };
use crate::rawmode::RawMode;
use crate::terminal::Terminal;
use anyhow::{ anyhow, Context };
use crate::key::Key;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    _mode: RawMode,
    stdin: std::io::Stdin,
    terminal: Terminal,

    insert_char_buffer: Vec<u8>,
    mode: Mode,

    lsp_client: Option<Arc<LspClient>>,
    buffers: Vec<Rc<RefCell<TextBuffer>>>,
    viewers: Vec<(TextViewer<TextBuffer>, ViewerRect)>,
    hover: HoverViewer,
    active: usize,
}

impl Editor {
    pub fn new() -> anyhow::Result<Editor> {
        let terminal = Terminal::new()?;
        let rect = ViewerRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 };
        let buffer = Rc::new(RefCell::new(TextBuffer::open("./test.txt")?));
        Ok(Editor {
            _mode: RawMode::enable_raw_mode().context("enable raw mode failed")?,
            stdin: std::io::stdin(),
            terminal,
            insert_char_buffer: vec![],
            mode: Mode::Normal,

            lsp_client: None,
            buffers: vec![buffer.clone()],
            viewers: vec![(TextViewer::open(buffer.clone())?, rect)],
            hover: HoverViewer::empty(),
            active: 0,
        })
    }

    pub async fn new_clangd() -> anyhow::Result<Editor> {
        let terminal = Terminal::new()?;
        let rect = ViewerRect { h: terminal.height(), w: terminal.width(), i: 0, j: 0 };

        let lsp_client = LspClient::start(LspClientStartArg { program: "clangd".to_owned() })?;
        {
            use lsp_types::*;
            let client_capabilities = ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities { hover: Some(HoverClientCapabilities { dynamic_registration: Some(true), content_format: Some(vec![MarkupKind::PlainText, MarkupKind::Markdown]) }), ..Default::default() }),
                ..Default::default()
            };

            let work = WorkspaceFolder {
                uri: path_to_uri("./")?,
                name: "test".to_owned(),
            };

            let init_params = InitializeParams {
                process_id: Some(std::process::id()),
                capabilities: client_capabilities,
                workspace_folders: Some(vec![work]),
                ..Default::default()
            };
            let recv = lsp_client.request::<lsp_types::request::Initialize>(init_params).await?;
            let _inited = recv.await_result().await?;
            lsp_client.notify::<notification::Initialized>(InitializedParams {}).await?;
        }

        let lsp_client = Arc::new(lsp_client);

        let buffer = Rc::new(RefCell::new(TextBuffer::open_with_lsp("./1.cpp", lsp_client.clone()).await?));
        Ok(Editor {
            _mode: RawMode::enable_raw_mode().context("enable raw mode failed")?,
            stdin: std::io::stdin(),
            terminal,
            insert_char_buffer: vec![],
            mode: Mode::Normal,

            lsp_client: Some(lsp_client),
            buffers: vec![buffer.clone()],
            viewers: vec![(TextViewer::open(buffer.clone())?, rect)],
            hover: HoverViewer::empty(),
            active: 0,
        })
    }

    fn update_all(&mut self) -> anyhow::Result<()> {
        self.terminal.clear_all()?;
        for (viewer, rect) in self.viewers.iter_mut() {
            viewer.draw_all(rect, &mut self.terminal)?;
        }
        self.hover.draw_all(&ViewerRect { h: self.terminal.height() / 2, w: self.terminal.width(), i: self.terminal.height() / 2, j: 0 }, &mut self.terminal)?;
        let active_rect = self.viewers[self.active].1.clone();
        self.viewers[self.active].0.draw_cursor(&active_rect, &mut self.terminal)?;
        self.terminal.flush()
    }

    async fn normal_input(&mut self, key: Key) -> anyhow::Result<()> {
             if key == Key::char(b'j') { self.viewers[self.active].0.move_down() }
        else if key == Key::char(b'k') { self.viewers[self.active].0.move_up() }
        else if key == Key::char(b'h') { self.viewers[self.active].0.move_left() }
        else if key == Key::char(b'l') { self.viewers[self.active].0.move_right() }
        else if key == Key::char(b'i') { self.mode = Mode::Insert; Ok(()) }
        else if key == Key::ctrl(b'w') { self.active = (self.active + 1) % self.viewers.len(); Ok(()) }
        else if key == Key::char(b'K') {
            let recv = self.viewers[self.active].0.hover().await?;
            self.hover = match recv {
                Some(recv) => {
                    let res = recv.await_result().await?.0?;
                    res
                        .map(|hover|
                             if let lsp_types::HoverContents::Markup(content) = hover.contents {
                                 HoverViewer::new(format!("{}", content.value))
                             }
                             else {
                                 HoverViewer::empty()
                             }
                        )
                        .unwrap_or(HoverViewer::empty())
                }
                None => HoverViewer::empty(),
            };
            Ok(())
        }
        else { Ok(()) }
    }

    fn insert_input(&mut self, key: Key) -> anyhow::Result<()> {
        if self.insert_char_buffer.len() > 0 {
            if let Key::Character(ch) = key {
                self.insert_char_buffer.push(ch);
            }
            else {
                return Err(anyhow!("bugged char?"))
            }
        }
        else if key == Key::escape() {
            self.mode = Mode::Normal;
        }
        else if key == Key::backspace() {
            self.viewers[self.active].0.backspace()?;
        }
        else if key == Key::char(b'\r') {
            self.viewers[self.active].0.newline()?;
        }
        else if let Key::Character(ch) = key {
            if ch >= 32 {
                self.insert_char_buffer.push(ch);
            }
        }
        if self.insert_char_buffer.len() > 0 {
            if let Ok(st) = String::from_utf8(self.insert_char_buffer.clone()) {
                for c in st.chars() {
                    self.viewers[self.active].0.insert_char(c)?;
                }
                self.insert_char_buffer.clear();
            }
        }
        Ok(())
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.update_all()?;
        loop {
            if let Some(key) = Key::try_read_from_stdin(&mut self.stdin)? {
                if key == Key::ctrl(b'c') {
                    break;
                }
                else if self.mode == Mode::Normal {
                    self.normal_input(key).await?;
                }
                else if self.mode == Mode::Insert {
                    self.insert_input(key)?;
                }
                self.update_all()?;
            }
        }
        Ok(())
    }
}

/*
impl Editor {
    pub fn multi_viewer_test() -> anyhow::Result<Self> {
        let terminal = Terminal::new()?;
        let w = terminal.width() / 2;
        let rect1 = ViewerRect { h: terminal.height(), w, i: 0, j: 0 };
        let rect2 = ViewerRect { h: terminal.height(), w, i: 0, j: w };
        let buffer = Rc::new(RefCell::new(TextBuffer::open("./test.txt")?));
        Ok(Editor {
            _mode: RawMode::enable_raw_mode().context("enable raw mode failed")?,
            stdin: std::io::stdin(),
            terminal,
            insert_char_buffer: vec![],
            mode: Mode::Normal,

            buffers: vec![buffer.clone()],
            viewers: vec![
                (Box::new(TextViewer::open(buffer.clone())?), rect1),
                (Box::new(TextViewer::open(buffer.clone())?), rect2),
            ],
            active: 0,
        })
    }
}
*/
