#![feature(async_await, await_macro)]

use dotenv::dotenv;
use env_logger;
use failure::Error;
use futures03::Future;
use log;
use std::env;
use tgbot::{
    handle_updates,
    methods::SendMessage,
    types::{Update, UpdateKind},
    Api, Config, UpdateHandler, UpdateMethod,
};

struct Handler {
    api: Api,
}

impl UpdateHandler for Handler {
    fn handle(&mut self, update: Update) {
        log::info!("got an update: {:?}\n", update);
        if let UpdateKind::Message(message) = update.kind {
            if let Some(text) = message.get_text() {
                let chat_id = message.get_chat_id();
                let method = SendMessage::new(chat_id, text.data.clone());
                let api = self.api.clone();
                self.api.spawn(
                    async {
                        let message = await!(api.execute(method))?;
                        log::info!("sendMessage result: {:?}\n", message);
                        Ok::<_, Error>(())
                    },
                );
            }
        }
    }
}

fn main() {
    dotenv().ok();
    env_logger::init();

    let token = env::var("TGRS_TOKEN").expect("TGRS_TOKEN is not set");
    let proxy = env::var("TGRS_PROXY").ok();
    let mut config = Config::new(token);
    if let Some(proxy) = proxy {
        config = config.proxy(proxy);
    }
    let api = Api::new(config).expect("Failed to create API");
    tokio::run(handle_updates(UpdateMethod::poll(api.clone()), Handler { api }));
}
