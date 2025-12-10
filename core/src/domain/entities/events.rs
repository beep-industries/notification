use serde::Deserialize;

#[derive(Deserialize)]
pub struct NotifyEntry {
    pub r#type: String,
    pub id: String,
}

#[derive(Deserialize)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub url: String,
}
// For from_value but maybe TODO remove
#[derive(Deserialize)]
pub struct CreateMessageEvent {
    pub message_id: String,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub reply_to_message_id: Option<String>,
    pub attachments: Vec<Attachment>,
    pub notify_entries: Vec<NotifyEntry>,
}

pub struct UpdateMessageEvent {
    pub message_id: String,
    pub content: String,
    pub is_pinned: Option<bool>,
    pub notify_entries: Vec<NotifyEntry>,
}

pub struct DeleteMessageEvent {
    pub message_id: String,
}

// impl From<serde_json::Value> for CreateMessageEvent {
//     fn from(value: serde_json::Value) -> Self {

//     }
// }