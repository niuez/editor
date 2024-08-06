use super::msg::{Message, Notification, Request, RequestId, Response, ResponseError};
use anyhow::{anyhow, Context};

use tokio::{sync::{Mutex, Notify, mpsc::{ self, Receiver, Sender }}, task::JoinHandle};

use std::{collections::HashMap, str::FromStr, sync::Arc};

pub struct LspClient {
    lsp_process_child: tokio::process::Child,
    from_server_thread: tokio::task::JoinHandle<anyhow::Result<()>>,
    from_server_receiver: Receiver<Message>,
    to_server_thread: tokio::task::JoinHandle<anyhow::Result<()>>,
    to_server_sender: Sender<Message>,

    response_senders: Arc<Mutex<HashMap<RequestId, tokio::sync::oneshot::Sender<Response>>>>,

    id_cnt: Mutex<i32>,
}

pub struct LspClientStartArg {
    pub program: String,
}

impl LspClient {
    pub fn start(start_arg: LspClientStartArg) -> anyhow::Result<Self> {
        let mut child = tokio::process::Command::new(start_arg.program)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::io::stderr())
            .spawn()
            .context("failed to launch")?;

        let mut to_server = child.stdin.take().unwrap();
        let from_server = child.stdout.take().unwrap();
        let mut server_reader = tokio::io::BufReader::new(from_server);

        let response_senders = Arc::new(Mutex::new(HashMap::<RequestId, tokio::sync::oneshot::Sender<Response>>::new()));

        let response_senders_for_thread = response_senders.clone();

        let (from_server_sender, from_server_receiver) = mpsc::channel::<Message>(1000);
        let from_server_thread =
            tokio::spawn(async move {
                while let Some(msg) = Message::read(&mut server_reader).await.context("message read failed")? {
                    match msg {
                        Message::Response(res) => {
                            eprintln!("got {:?}", res);
                            let opt_sender = {
                                response_senders_for_thread.as_ref().lock().await.remove(&res.id)
                            };
                            eprintln!("opt_sender: {:?}", opt_sender);
                            if let Some(sender) = opt_sender {
                                sender.send(res).map_err(|_e| anyhow!("receiver dropped"))?;
                            }
                        }
                        Message::Request(req) => {
                        }
                        Message::Notification(ntf) => {
                        }
                    }
                    eprintln!("read time");
                }
                Ok(())
            });

        let (to_server_sender, mut to_server_receiver) = mpsc::channel::<Message>(1000);
        let to_server_thread = 
            tokio::spawn(async move {
                while let Some(it) = to_server_receiver.recv().await {
                    it.write(&mut to_server).await.context("to server failed")?
                }
                Ok(())
            });

        Ok(Self {
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

    pub async fn request<R: lsp_types::request::Request>(&self, param: R::Params) -> anyhow::Result<ResponseReceiver<R>>
    where R::Params: Clone
    {
        let (sender, receiver) = tokio::sync::oneshot::channel::<Response>();
        let id = self.get_new_id().await;
        {
            self.response_senders.as_ref().lock().await.insert(id.clone(), sender);
        }

        let req = Request::new(id, R::METHOD.to_owned(), param.clone());
        let msg = Message::Request(req);
        self.to_server_sender.send(msg).await?;

        let (sender2, receiver2) = tokio::sync::oneshot::channel::<ResponseResult<R>>();

        let handle = tokio::spawn(async move {
            match receiver.await {
                Ok(resp) => {
                    sender2.send(response_to_result::<R>(resp)).map_err(|_e| anyhow!("receiver2 dropped"))
                }
                Err(_) => {
                    Err(anyhow!("sender dropped"))
                }
            }
        });
        Ok(ResponseReceiver { receiver: receiver2, handle, param })
    }

    pub async fn notify<N: lsp_types::notification::Notification>(&self, param: N::Params) -> anyhow::Result<()> {
        let nt = Notification::new(N::METHOD.to_owned(), param);
        let msg = Message::Notification(nt);
        self.to_server_sender.send(msg).await?;
        Ok(())
    }
}

pub struct ResponseReceiver<R: lsp_types::request::Request> {
    pub receiver: tokio::sync::oneshot::Receiver<ResponseResult<R>>,
    handle: JoinHandle<anyhow::Result<()>>,
    pub param: R::Params,
}

impl<R: lsp_types::request::Request> ResponseReceiver<R> {
    pub fn abort_request(self) {
        self.handle.abort();
    }

    pub async fn await_result(self) -> anyhow::Result<(ResponseResult<R>, R::Params)> {
        Ok((self.receiver.await?, self.param))
    }
    pub fn try_get_response(mut self) -> TryGetResponse<R> {
        match self.receiver.try_recv() {
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => TryGetResponse::Yet(self),
            Ok(resp) => TryGetResponse::Receive((resp, self.param)),
            Err(tokio::sync::oneshot::error::TryRecvError::Closed) => unreachable!("closed???")

        }
    }
}

pub enum TryGetResponse<R: lsp_types::request::Request> {
    Receive((ResponseResult<R>, R::Params)),
    Yet(ResponseReceiver<R>),
}

pub type ResponseResult<R> = anyhow::Result<<R as lsp_types::request::Request>::Result>;


fn response_to_result<R: lsp_types::request::Request>(resp: Response) -> ResponseResult<R> {
    match resp.error {
        None => {
            match resp.result {
                Some(r) => Ok(serde_json::from_value(r)?),
                None => unreachable!(),
            }
        }
        Some(e) => Err(e.into())
    }
}

pub fn path_to_uri<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<lsp_types::Uri> {
    Ok(lsp_types::Uri::from_str(&format!("file://{}", std::path::absolute(path)?.to_str().context("cant to_str")?))?)
}
