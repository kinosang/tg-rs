#![feature(async_await, await_macro)]

use dotenv::dotenv;
use env_logger;
use failure::Error;
use futures01::{sync::oneshot::channel, Future, Stream};
use futures03::{compat::Future01CompatExt, TryFutureExt};
use hyper::{header::HeaderValue, Body, Client, Method, Request, Server, StatusCode};
use log;
use tgbot::{types::Update, UpdateHandler, WebhookServiceFactory};
use tokio::runtime::current_thread::block_on_all;

struct Handler;

impl UpdateHandler for Handler {
    fn handle(&mut self, update: Update) {
        log::debug!("got an update: {:?}\n", update);
    }
}

#[test]
fn webhook() {
    dotenv().ok();
    env_logger::init();
    let (tx, rx) = channel::<()>();
    let server = Server::bind(&([127, 0, 0, 1], 8080).into())
        .serve(WebhookServiceFactory::new("/", Handler))
        .with_graceful_shutdown(rx)
        .map_err(|e| log::error!("Server error: {}", e));
    let (status, body) = block_on_all(
        Box::pin(
            async {
                tokio::spawn(server);
                let client = Client::new();
                let json: serde_json::Value = serde_json::json!({
                    "update_id": 10000,
                    "message": {
                        "date": 1_441_645_532,
                        "chat": {
                            "last_name": "Test Lastname",
                            "id": 1_111_111,
                            "first_name": "Test",
                            "username": "Test",
                            "type": "private"
                        },
                        "message_id": 1365,
                        "from": {
                            "last_name": "Test Lastname",
                            "id": 1_111_111,
                            "first_name": "Test",
                            "username": "Test",
                            "is_bot": false
                        },
                        "text": "/start"
                    }
                });
                let json = json.to_string();
                let uri: hyper::Uri = "http://localhost:8080/".parse().unwrap();

                let mut req = Request::new(Body::from(json));
                *req.method_mut() = Method::POST;
                *req.uri_mut() = uri.clone();
                req.headers_mut().insert(
                    hyper::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );

                let resp = await!(client.request(req).compat())?;
                let _ = tx.send(());
                let status = resp.status();
                let body = await!(resp.into_body().concat2().compat())?;
                let body = String::from_utf8(body.into_iter().collect())?;
                Ok::<_, Error>((status, body))
            },
        )
        .compat(),
    )
    .unwrap();
    log::debug!("Webhook response body: {:?}", body);
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "");
}
