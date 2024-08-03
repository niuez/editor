use std::io::{self, BufRead, BufReader, Error, ErrorKind, Read, Stdin, Stdout, Write};
use anyhow::Context;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Key {
    Character(u8),
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
}

impl Key {
    pub fn char(c: u8) -> Self { Key::Character(c) }
    pub fn ctrl(c: u8) -> Self { Key::Character(c & 0x1f) }
    pub fn backspace() -> Self { Key::Character(127) }
    pub fn escape() -> Self { Key::Character(27) }

    pub fn try_read_from_stdin(stdin: &mut Stdin) -> anyhow::Result<Option<Key>> {
        let mut buf = [0; 1];
        if read_non_blocking(stdin, &mut buf)? == 1 {
            Ok(Some(
                match buf[0] {
                    b'\x1b' => read_escape_sequence(stdin)?,
                    ch => Key::Character(ch),
                }
            ))
        }
        else {
            Ok(None)
        }

    }
}

fn read_non_blocking<R: Read>(r: &mut R, buf: &mut [u8]) -> anyhow::Result<usize> {
    r.read(buf)
        .or_else(|e| {
            if e.kind() == ErrorKind::WouldBlock {
                Ok(0)
            } else {
                Err(e)
            }
        }).context("read non blocking error")
}

fn read_escape_sequence(stdin: &mut Stdin) -> anyhow::Result<Key> {
    let mut seq = [0; 2];
    let n = read_non_blocking(stdin, &mut seq)?;
    if n == 2 && seq[0] == b'[' {
        if seq[1] >= b'0' && seq[1] <= b'9' {
            let mut last = [0; 1];
            if read_non_blocking(stdin, &mut last)? == 1 && last[0] == b'~' {
                Ok(
                    match seq[1] {
                        b'1' | b'7' => Key::Home,
                        b'3' => Key::Delete,
                        b'4' | b'8' => Key::End,
                        b'5' => Key::PageUp,
                        b'6' => Key::PageDown,
                        _ => Key::Character(b'\x1b'),
                    }
                )
            } else {
                Ok(Key::Character(b'\x1b'))
            }
        } else {
            Ok(
                match seq[1] {
                    b'A' => Key::ArrowUp,
                    b'B' => Key::ArrowDown,
                    b'C' => Key::ArrowRight,
                    b'D' => Key::ArrowLeft,
                    b'H' => Key::Home,
                    b'F' => Key::End,
                    _ => Key::Character(b'\x1b'),
                }
            )
        }
    } else if n == 2 && seq[0] == b'O' {
        Ok(
            match seq[1] {
                b'H' => Key::Home,
                b'F' => Key::End,
                _ => Key::Character(b'\x1b'),
            }
        )
    } else {
        Ok(Key::Character(b'\x1b'))
    }
}
