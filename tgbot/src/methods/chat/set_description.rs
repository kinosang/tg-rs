use crate::{methods::Method, request::RequestBuilder, types::ChatId};
use failure::Error;
use serde::Serialize;

/// Change the description of a group, a supergroup or a channel
///
/// The bot must be an administrator in the chat for this to work
/// and must have the appropriate admin rights
#[derive(Clone, Debug, Serialize)]
pub struct SetChatDescription {
    chat_id: ChatId,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl SetChatDescription {
    /// Creates a new SetChatDescription
    ///
    /// # Arguments
    ///
    /// * chat_id - Unique identifier for the target chat
    pub fn new<C: Into<ChatId>>(chat_id: C) -> Self {
        SetChatDescription {
            chat_id: chat_id.into(),
            description: None,
        }
    }

    /// New chat description, 0-255 characters
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl Method for SetChatDescription {
    type Response = bool;

    fn into_request(self) -> Result<RequestBuilder, Error> {
        RequestBuilder::json("setChatDescription", &self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{RequestBody, RequestMethod};
    use serde_json::Value;

    #[test]
    fn set_chat_description() {
        let request = SetChatDescription::new(1)
            .description("desc")
            .into_request()
            .unwrap()
            .build("base-url", "token");
        assert_eq!(request.method, RequestMethod::Post);
        assert_eq!(request.url, "base-url/bottoken/setChatDescription");
        if let RequestBody::Json(data) = request.body {
            let data: Value = serde_json::from_slice(&data).unwrap();
            assert_eq!(data["chat_id"], 1);
            assert_eq!(data["description"], "desc");
        } else {
            panic!("Unexpected request body: {:?}", request.body);
        }
    }
}
