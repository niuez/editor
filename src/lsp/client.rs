use super::msg::{Message, Notification, Request, RequestId, Response};
use anyhow::Context;

use tokio::sync::{Mutex, Notify, mpsc::{ self, Receiver, Sender }};

use std::{cell::RefCell, collections::HashMap, sync::Arc};

pub struct Client {
    lsp_process_child: std::process::Child,
    from_server_thread: tokio::task::JoinHandle<anyhow::Result<()>>,
    from_server_receiver: Receiver<Message>,
    to_server_thread: tokio::task::JoinHandle<anyhow::Result<()>>,
    to_server_sender: Sender<Message>,

    response_senders: Arc<Mutex<HashMap<RequestId, tokio::sync::oneshot::Sender<Response>>>>,

    id_cnt: Mutex<i32>,
}

pub struct ClientStartArg {
    program: String,
}

impl Client {
    pub fn start(start_arg: ClientStartArg) -> anyhow::Result<Self> {
        let mut child = std::process::Command::new(start_arg.program)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::io::stderr())
            .spawn()
            .context("failed to launch")?;

        let mut to_server = child.stdin.take().unwrap();
        let from_server = child.stdout.take().unwrap();
        let mut server_reader = std::io::BufReader::new(from_server);

        let response_senders = Arc::new(Mutex::new(HashMap::<RequestId, tokio::sync::oneshot::Sender<Response>>::new()));

        let response_senders_for_thread = response_senders.clone();

        let (from_server_sender, from_server_receiver) = mpsc::channel::<Message>(1000);
        let from_server_thread =
            tokio::spawn(async move {
                while let Some(msg) = Message::read(&mut server_reader).context("message read failed")? {
                    match msg {
                        Message::Response(res) => {
                            let opt_sender = {
                                response_senders_for_thread.as_ref().lock().await.remove(&res.id)
                            };
                            if let Some(sender) = opt_sender {
                                sender.send(res).unwrap();
                            }
                        }
                        Message::Request(req) => {
                        }
                        Message::Notification(ntf) => {
                        }
                    }
                }
                Ok(())
            });

        let (to_server_sender, mut to_server_receiver) = mpsc::channel::<Message>(1000);
        let to_server_thread = 
            tokio::spawn(async move {
                while let Some(it) = to_server_receiver.recv().await {
                    it.write(&mut to_server).context("to server failed")?
                }
                Ok(())
            });

        Ok(Client {
            lsp_process_child: child,
            from_server_thread,
            from_server_receiver,
            to_server_thread,
            to_server_sender,
            response_senders,
            id_cnt: Mutex::new(0),
        })
    }

    async fn get_new_id(&self) -> RequestId {
        let mut num = self.id_cnt.lock().await;
        let ans = *num;
        *num += 1;
        RequestId::from(ans)
    }

    pub async fn request<R: lsp_types::request::Request>(&self, param: R::Params) -> anyhow::Result<tokio::sync::oneshot::Receiver<Response>> {
        let (sender, receiver) = tokio::sync::oneshot::channel::<Response>();
        let id = self.get_new_id().await;
        {
            self.response_senders.as_ref().lock().await.insert(id.clone(), sender);
        }

        let req = Request::new(id, R::METHOD.to_owned(), param);
        let msg = Message::Request(req);
        self.to_server_sender.send(msg).await?;
        Ok(receiver)
    }

    pub async fn notify<N: lsp_types::notification::Notification>(&self, param: N::Params) -> anyhow::Result<()> {
        let nt = Notification::new(N::METHOD.to_owned(), param);
        let msg = Message::Notification(nt);
        self.to_server_sender.send(msg).await?;
        Ok(())
    }
}
