use lsp_types::{CompletionResponse, CompletionTextEdit};
use ropey::Rope;
use crate::{buffer::{CursorPos, text_buffer::TextBuffer, Buffer}, terminal::Terminal};
use super::{Draw, ViewerRect};

pub struct CompletionViewer {
    rope: Rope,
    x: usize,
    select: usize,
    resp: CompletionResponse,
    len: usize,
    pub cursor: CursorPos,
}

impl CompletionViewer {
    pub fn new(mut resp: CompletionResponse, cursor: CursorPos) -> Self {
        let (text, len) = match resp {
            CompletionResponse::Array(ref mut arr) => {
                arr.sort_by_cached_key(|i| i.sort_text.to_owned());
                (arr.iter().map(|a| a.label.to_owned()).collect::<Vec<_>>().join("\n"), arr.len())
            }
            CompletionResponse::List(ref mut list) => {
                list.items.sort_by_cached_key(|i| i.sort_text.to_owned());
                (list.items.iter().map(|a| a.label.to_owned()).collect::<Vec<_>>().join("\n"), list.items.len())
            }
        };
        Self {
            rope: Rope::from_str(&text),
            x: 0,
            select: 0,
            resp,
            len,
            cursor,
        }
    }

    pub async fn do_completion<B: Buffer>(&self, buffer: &mut B) -> anyhow::Result<CursorPos> {
        let item = match self.resp {
            CompletionResponse::Array(ref arr) => {
                arr.get(self.select)
            }
            CompletionResponse::List(ref list) => {
                list.items.get(self.select)
            }
        };
        eprintln!("complete = {:?}", item);
        if let Some(item) = item {
            let cursor = match item.text_edit.as_ref() {
                Some(CompletionTextEdit::Edit(edit)) => {
                    buffer.edit((edit.range.start.line as usize, edit.range.start.character as usize), (edit.range.end.line as usize, edit.range.end.character as usize), &edit.new_text).await?
                }
                None => { self.cursor }
                _ => unimplemented!(),
            };

            /*
            if let Some(edits) = item.additional_text_edits.as_ref() {
                for edit in edits {
                    buffer.edit((edit.range.start.line as usize, edit.range.start.character as usize), (edit.range.end.line as usize, edit.range.end.character as usize), &edit.new_text).await?;
                }
            }
            */
            Ok(cursor)
        }
        else {
            Ok(self.cursor)
        }
    }

    pub fn select_next(&mut self) {
        self.select = (self.select + 1) % self.len;
    }

    pub fn select_prev(&mut self) {
        self.select = (self.select + self.len - 1) % self.len;
    }

    fn fix_top(&mut self, rect: &ViewerRect) {
        if self.x > self.select {
            self.x = self.select
        }
        if self.select >= self.x + rect.h {
            self.x = self.select - rect.h + 1;
        }
    }
}

impl Draw for CompletionViewer {
    fn draw_all(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        self.fix_top(rect);
        for i in 0..rect.h {
            if let Some(slice) = self.rope.get_line(self.x + i) {
                let len = slice.len_chars();
                terminal.set_cursor(rect.i + i, rect.j)?;
                if 0 < len {
                    terminal.write(format!("{}{}",
                        if self.select == self.x + i { ">" } else { " " },
                        slice.slice(0..(len - 1).min(0 + rect.w))).as_bytes()
                    )?;
                }
            }
        }
        Ok(())
    }
    fn draw_cursor(&mut self, rect: &ViewerRect, terminal: &mut Terminal) -> anyhow::Result<()> {
        Ok(())
    }
}


