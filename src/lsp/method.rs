use super::client::{LspClient, ResponseReceiver, TryGetResponse};

pub mod hover;
pub mod didchange;
pub mod completion;

pub trait LspParam {
    type ActualParam;
    fn into_param(self) -> Self::ActualParam;
}

pub trait LspResult {
    type Response;
    type Param;
    fn from_response(resp: Self::Response, param: Self::Param) -> Self;
}

pub enum LspFetch<Request: lsp_types::request::Request, Result> {
    Yet(ResponseReceiver<Request>),
    Got(Result),
    Tmp,
}

impl<Request, Res> LspFetch<Request, Res>
where
    Request: lsp_types::request::Request,
    <Request as lsp_types::request::Request>::Params: Clone,
    Res: LspResult<Response=<Request as lsp_types::request::Request>::Result, Param=<Request as lsp_types::request::Request>::Params>,
{
    pub async fn new<P: LspParam<ActualParam=<Request as lsp_types::request::Request>::Params>>(client: &LspClient, param: P) -> anyhow::Result<Self> {
        let params = param.into_param();
        let receiver = client.request::<Request>(params).await?;
        Ok(Self::Yet(receiver))
    }

    pub fn abort(self) {
        if let Self::Yet(receiver) = self {
            receiver.abort_request();
        }
    }


    pub async fn await_result(self) -> anyhow::Result<Res> {
        match self {
            Self::Yet(receiver) => {
                let (resp, param) = receiver.await_result().await?;
                resp.map(|resp| Res::from_response(resp, param))
            }
            Self::Got(r) => Ok(r),
            _ => unreachable!(),
        }
    }

    pub fn try_get_result(&mut self) -> anyhow::Result<Option<&Res>> {
        let mut v = std::mem::replace(self, Self::Tmp);
        v = match v {
            Self::Yet(receiver) => {
                match receiver.try_get_response() {
                    TryGetResponse::Yet(receiver) => Self::Yet(receiver),
                    TryGetResponse::Receive(resp) => Self::Got(resp.0.map(|r| Res::from_response(r, resp.1))?),
                }
            }
            Self::Got(r) => Self::Got(r),
            _ => unreachable!(),
        };
        *self = v;
        Ok(match self {
            Self::Yet(_) => None,
            Self::Got(ref r) => Some(r),
            _ => unreachable!(),
        })
    }

}

