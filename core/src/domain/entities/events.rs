use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct NotifyEntry {
    pub r#type: String,
    pub id: String,
}

#[derive(Deserialize, Serialize)]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub url: String,
}

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

#[derive(Deserialize)]
pub struct UpdateMessageEvent {
    pub message_id: String,
    pub content: String,
    pub is_pinned: Option<bool>,
    pub notify_entries: Vec<NotifyEntry>,
}

#[derive(Deserialize)]
pub struct DeleteMessageEvent {
    pub message_id: String,
}