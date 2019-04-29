use crate::{
    executor::{default_executor, proxy_executor, Executor},
    methods::Method,
    request::RequestBuilder,
    types::Response,
};
use failure::Error;
use futures03::{Future, TryFutureExt};
use serde::de::DeserializeOwned;
use std::{fmt::Debug, sync::Arc};

const DEFAULT_HOST: &str = "https://api.telegram.org";

/// An API config
#[derive(Debug, Clone)]
pub struct Config {
    host: Option<String>,
    token: String,
    proxy: Option<String>,
}

impl Config {
    /// Creates a new config with given token
    pub fn new<S: Into<String>>(token: S) -> Self {
        Self {
            token: token.into(),
            host: None,
            proxy: None,
        }
    }

    /// Sets an API host
    ///
    /// https://api.telegram.org is used by default
    pub fn host<S: Into<String>>(mut self, host: S) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets a proxy to config
    ///
    /// Proxy format:
    /// * http://\[user:password\]@host:port
    /// * https://\[user:password\]@host:port
    /// * socks4://userid@host:port
    /// * socks5://\[user:password\]@host:port
    pub fn proxy<S: Into<String>>(mut self, proxy: S) -> Self {
        self.proxy = Some(proxy.into());
        self
    }
}

impl<S> From<S> for Config
where
    S: Into<String>,
{
    fn from(token: S) -> Self {
        Config::new(token.into())
    }
}

/// Telegram Bot API client
#[derive(Clone)]
pub struct Api {
    executor: Arc<Box<Executor>>,
    host: String,
    token: String,
}

impl Api {
    /// Creates a new client
    pub fn new<C: Into<Config>>(config: C) -> Result<Self, Error> {
        let config = config.into();
        Ok(Api {
            executor: Arc::new(if let Some(ref proxy) = config.proxy {
                proxy_executor(proxy)?
            } else {
                default_executor()?
            }),
            host: config.host.unwrap_or_else(|| String::from(DEFAULT_HOST)),
            token: config.token,
        })
    }

    /// Downloads a file
    ///
    /// Use getFile method in order get file_path
    #[allow(clippy::needless_lifetimes)]
    pub async fn download_file<P: AsRef<str>>(&self, file_path: P) -> Result<Vec<u8>, Error> {
        let executor = self.executor.clone();
        let req = RequestBuilder::empty(file_path.as_ref())
            .map(|builder| builder.build(format!("{}/file", &self.host), &self.token))?;
        let vec = await!(executor.execute(req))?;
        Ok(vec)
    }

    /// Executes a method
    pub fn execute<M: Method>(&self, method: M) -> impl Future<Output = Result<M::Response, Error>>
    where
        M::Response: DeserializeOwned + Send + 'static,
    {
        let host = self.host.clone();
        let token = self.token.clone();
        let executor = self.executor.clone();
        async move {
            let req = method.into_request().map(|builder| builder.build(host, token))?;
            let data = await!(executor.execute(req))?;
            let resp = serde_json::from_slice::<Response<M::Response>>(&data)?;
            match resp {
                Response::Success(obj) => Ok(obj),
                Response::Error(err) => Err(err.into()),
            }
        }
    }

    /// Spawns a future on the default executor.
    pub fn spawn<F, T, E: Debug>(&self, f: F)
    where
        F: Future<Output = Result<T, E>> + 'static + Send,
    {
        tokio_executor::spawn(
            Box::pin(
                async {
                    if let Err(e) = await!(f) {
                        log::error!("An error has occurred: {:?}", e)
                    }
                    Ok(())
                },
            )
            .compat(),
        );
    }
}
