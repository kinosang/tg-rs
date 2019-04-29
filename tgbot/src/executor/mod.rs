mod hyper;

pub(crate) use self::hyper::{default_executor, proxy_executor};
use std::pin::Pin;

use crate::request::Request;
use failure::Error;
use futures03::Future;

pub(crate) trait Executor: Send + Sync {
    fn execute(&self, req: Request) -> Pin<Box<Future<Output = Result<Vec<u8>, Error>> + Send>>;
}
