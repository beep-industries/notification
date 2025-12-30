#[cfg(test)]
mod tests {
    use crate::domain::{
        entities::events::{Attachment, CreateMessageEvent, NotifyEntry},
        generate_id,
    };

    #[test]
    fn test_create_message_event_serialization() {
        let event = CreateMessageEvent {
            message_id: generate_id().to_string(),
            author_id: generate_id().to_string(),
            channel_id: generate_id().to_string(),
            content: "Test content".to_string(),
            reply_to_message_id: None,
            attachments: vec![],
            notify_entries: vec![],
        };

        let json = serde_json::to_string(&event).expect("Should serialize");
        let deserialized: CreateMessageEvent =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.content, event.content);
        assert_eq!(deserialized.message_id, event.message_id);
    }

    #[test]
    fn test_create_message_event_with_attachments_and_notify_entries() {
        let event = CreateMessageEvent {
            message_id: generate_id().to_string(),
            author_id: generate_id().to_string(),
            channel_id: generate_id().to_string(),
            content: "Message with attachments".to_string(),
            reply_to_message_id: None,
            attachments: vec![Attachment {
                id: generate_id().to_string(),
                name: "file.pdf".to_string(),
                url: "https://example.com/file.pdf".to_string(),
            }],
            notify_entries: vec![NotifyEntry {
                r#type: "user".to_string(),
                id: generate_id().to_string(),
            }],
        };

        let json = serde_json::to_string(&event).expect("Should serialize");
        let deserialized: CreateMessageEvent =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.attachments.len(), 1);
        assert_eq!(deserialized.attachments[0].name, "file.pdf");
        assert_eq!(deserialized.notify_entries.len(), 1);
        assert_eq!(deserialized.notify_entries[0].r#type, "user");
    }

    #[test]
    fn test_event_serialization_preserves_content() {
        let original_content = "Test message with special chars: éàù";
        let event = CreateMessageEvent {
            message_id: generate_id().to_string(),
            author_id: generate_id().to_string(),
            channel_id: generate_id().to_string(),
            content: original_content.to_string(),
            reply_to_message_id: None,
            attachments: vec![],
            notify_entries: vec![],
        };

        let json = serde_json::to_string(&event).expect("Should serialize");
        let deserialized: CreateMessageEvent =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.content, original_content);
    }

    #[test]
    fn test_multiple_attachments_serialization() {
        let event = CreateMessageEvent {
            message_id: generate_id().to_string(),
            author_id: generate_id().to_string(),
            channel_id: generate_id().to_string(),
            content: "Multiple attachments".to_string(),
            reply_to_message_id: None,
            attachments: vec![
                Attachment {
                    id: generate_id().to_string(),
                    name: "doc1.pdf".to_string(),
                    url: "https://example.com/doc1.pdf".to_string(),
                },
                Attachment {
                    id: generate_id().to_string(),
                    name: "doc2.docx".to_string(),
                    url: "https://example.com/doc2.docx".to_string(),
                },
            ],
            notify_entries: vec![
                NotifyEntry {
                    r#type: "user".to_string(),
                    id: generate_id().to_string(),
                },
                NotifyEntry {
                    r#type: "team".to_string(),
                    id: "engineers".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&event).expect("Should serialize");
        let deserialized: CreateMessageEvent =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.attachments.len(), 2);
        assert_eq!(deserialized.attachments[0].name, "doc1.pdf");
        assert_eq!(deserialized.attachments[1].name, "doc2.docx");
        assert_eq!(deserialized.notify_entries.len(), 2);
        assert_eq!(deserialized.notify_entries[0].r#type, "user");
        assert_eq!(deserialized.notify_entries[1].r#type, "team");
    }

    #[test]
    fn test_create_message_event_missing_fields_fails() {
        let json = r#"{"message_id":"123","content":"test"}"#;
        let result: Result<CreateMessageEvent, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_with_reply_preserves_reply_id() {
        let reply_id = generate_id().to_string();
        let event = CreateMessageEvent {
            message_id: generate_id().to_string(),
            author_id: generate_id().to_string(),
            channel_id: generate_id().to_string(),
            content: "This is a reply".to_string(),
            reply_to_message_id: Some(reply_id.clone()),
            attachments: vec![],
            notify_entries: vec![],
        };

        let json = serde_json::to_string(&event).expect("Should serialize");
        let deserialized: CreateMessageEvent =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.reply_to_message_id, Some(reply_id));
    }
}
