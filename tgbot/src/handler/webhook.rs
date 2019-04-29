use crate::{types::Update, Never, UpdateHandler};
use futures01::{Future as Future01, Sink, Stream as _};
use futures03::{compat::Future01CompatExt, TryFutureExt};
use hyper::{
    header::{HeaderValue, ALLOW},
    service::{MakeService, Service},
    Body, Chunk, Error, Method, Request, Response, StatusCode,
};
use lazy_queue::sync::bounded::LazyQueue;
use tokio_executor::spawn;

/// Creates a webhook service
pub struct WebhookServiceFactory {
    path: String,
    queue: LazyQueue<Update>,
    processor: Option<Box<dyn Future01<Item = (), Error = ()> + Send>>,
}

impl WebhookServiceFactory {
    /// Creates a new factory
    pub fn new<S, H>(path: S, mut update_handler: H) -> WebhookServiceFactory
    where
        S: Into<String>,
        H: UpdateHandler + Send + Sync + 'static,
    {
        const QUEUE_SIZE: usize = 10;
        let (queue, processor) = LazyQueue::new(
            move |update| {
                update_handler.handle(update);
                Ok::<_, Never>(())
            },
            QUEUE_SIZE,
        );
        WebhookServiceFactory {
            path: path.into(),
            queue,
            processor: Some(Box::new(processor.map_err(|e| log::error!("Processing error: {}", e)))),
        }
    }
}

impl<Ctx> MakeService<Ctx> for WebhookServiceFactory {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Service = WebhookService;
    type Future = Box<Future01<Item = Self::Service, Error = Self::MakeError> + Send>;
    type MakeError = Never;

    fn make_service(&mut self, _ctx: Ctx) -> Self::Future {
        let path = self.path.clone();
        let queue = self.queue.clone();
        if let Some(fut) = self.processor.take() {
            spawn(fut);
        }
        Box::new(futures01::future::ok(WebhookService { path, queue }))
    }
}

/// Webhook service
pub struct WebhookService {
    path: String,
    queue: LazyQueue<Update>,
}

async fn put_on_a_queue(request: Request<Body>, queue: impl Sink<SinkItem = Update>) -> Result<Response<Body>, Error> {
    let body: Chunk = await!(request.into_body().concat2().compat())?;
    match serde_json::from_slice(&body) {
        Ok(update) => {
            let res = await!(queue.send(update).compat());
            if res.is_err() {
                log::warn!("The receiving end has been dropped");
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .expect("Can't construct an INTERNAL_SERVER_ERROR response"))
            } else {
                Ok(Response::new(Body::empty()))
            }
        }
        Err(err) => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(err.to_string()))
            .expect("Can't construct a BAD_REQUEST response")),
    }
}

impl Service for WebhookService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = Box<Future01<Item = Response<Body>, Error = Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        if let Method::POST = *req.method() {
            if req.uri().path() == self.path {
                Box::new(Box::pin(put_on_a_queue(req, self.queue.clone())).compat())
            } else {
                Box::new(futures01::future::ok(
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .expect("Can't construct a NOT_FOUND response"),
                ))
            }
        } else {
            Box::new(futures01::future::ok(
                Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .header(ALLOW, HeaderValue::from_static("POST"))
                    .body(Body::empty())
                    .expect("Can't construct a METHOD_NOT_ALLOWED response"),
            ))
        }
    }
}
