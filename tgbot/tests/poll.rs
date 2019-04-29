use dotenv::dotenv;
use futures03::{TryFutureExt, TryStreamExt};
use mockito::{mock, server_url, Matcher};
use serde_json::json;
use tgbot::prelude::*;
use tokio::runtime::current_thread::block_on_all;

#[test]
fn poll() {
    dotenv().ok();
    env_logger::init();
    let _m = mock("POST", "/bottoken/getUpdates")
        .match_body(Matcher::Json(json!({
            "offset": 1,
            "limit": 100,
            "timeout": 10,
            "allowed_updates": []
        })))
        .with_body(
            &serde_json::to_vec(&json!({
                "ok": true,
                "result": [{
                    "update_id": 1,
                    "message": {
                        "message_id": 1,
                        "date": 0,
                        "from": {
                            "id": 1,
                            "is_bot": false,
                            "first_name": "test"
                        },
                        "chat": {
                            "id": 1,
                            "type": "private",
                            "first_name": "test"
                        },
                        "text": "test"
                    }
                }]
            }))
            .unwrap(),
        )
        .create();
    let api = Api::new(Config::new("token").host(server_url())).unwrap();
    let mut fut = UpdatesStream::from(api).should_retry(false);
    let next = fut.try_next().compat();
    let update = block_on_all(next).unwrap().unwrap();
    assert_eq!(update.id, 1);
}
