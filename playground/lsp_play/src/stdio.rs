/*

use std::{
    io::{self},
    thread,
};

use crossbeam_channel::{bounded, Receiver, Sender};

use crate::msg::Message;

/// Creates an LSP connection via stdio.
pub(crate) fn stdio_transport<I: Send + std::io::Write, O: Send + std::io::Read>(mut server_stdin: I, mut server_stdout: O) -> (Sender<Message>, Receiver<Message>, IoThreads) {
    let (writer_sender, writer_receiver) = bounded::<Message>(0);
    let writer = thread::Builder::new()
        .name("LspServerWriter".to_owned())
        .spawn(move || {
            let mut server_stdin = server_stdin;
            writer_receiver.into_iter().try_for_each(|it| it.write(&mut server_stdin))
        })
        .unwrap();
    let (reader_sender, reader_receiver) = bounded::<Message>(0);
    let reader = thread::Builder::new()
        .name("LspServerReader".to_owned())
        .spawn(move || {
            let mut bufreader = std::io::BufReader::new(server_stdout);
            while let Some(msg) = Message::read(&mut bufreader)? {
                let is_exit = matches!(&msg, Message::Notification(n) if n.is_exit());

                eprintln!("sending message {:#?}", msg);
                reader_sender.send(msg).expect("receiver was dropped, failed to send a message");

                if is_exit {
                    break;
                }
            }
            Ok(())
        })
        .unwrap();
    let threads = IoThreads { reader, writer };
    (writer_sender, reader_receiver, threads)
}

// Creates an IoThreads
pub(crate) fn make_io_threads(
    reader: thread::JoinHandle<io::Result<()>>,
    writer: thread::JoinHandle<io::Result<()>>,
) -> IoThreads {
    IoThreads { reader, writer }
}

pub struct IoThreads {
    reader: thread::JoinHandle<io::Result<()>>,
    writer: thread::JoinHandle<io::Result<()>>,
}

impl IoThreads {
    pub fn join(self) -> io::Result<()> {
        match self.reader.join() {
            Ok(r) => r?,
            Err(err) => {
                println!("reader panicked!");
                std::panic::panic_any(err)
            }
        }
        match self.writer.join() {
            Ok(r) => r,
            Err(err) => {
                println!("writer panicked!");
                std::panic::panic_any(err);
            }
        }
    }
}
*/
