# RabbitMQ

It is assumed that we are going to use RabbitMQ queues, not streams, to simplify this first iteration. If the need gets identified, we may switch to streams.

## Consumers

This service listens for incoming messages, mentions and friend requests in order to store notifications. A separate service handles sending notifications to users via WebSocket (including DND, sound, popup logic).

### ConsumeMessageCreated

```
queue: notifications.messages.created
exchange name and type: messages.events of type Topic
binding: messages.created
```

**Purpose:** When a new message is created in the Messages service, create notification entries based on the `notify_entries` field.

**Context:** Can be either a DM or a server message.

**Message Schema (Proto):**

```protobuf
message CreateMessageEvent {
  string message_id = 1;          // ID of the newly created message
  string channel_id = 2;          // ID of the channel where the message was posted
  string author_id = 3;           // ID of the user who authored the message
  string content = 4;             // Content of the message
  string reply_to_message_id = 5; // ID of the message being replied to, if any
  message Attachment {
    string id = 1;
    string name = 2;
    string url = 3;
  }
  repeated Attachment attachments = 6;
  repeated NotifyEntry notify_entries = 7; // Entries to notify (users/roles)
}
```

**Storage Behavior:**

- For each entry in `notify_entries`, create a notification entry with:
  - Reference to `message_id`, `channel_id`, `author_id`, and `content`
  - Mark as unread
  - Store creation timestamp

### ConsumeMessageUpdated

```
queue: notifications.messages.updated
exchange name and type: messages.events of type Topic
binding: messages.updated
```

**Purpose:** When an existing message is modified, update the stored notification WITHOUT triggering a new notification.

**Message Schema (Proto):**

```protobuf
message UpdateMessageEvent {
  string message_id = 1;        // ID of the message being updated
  string content = 2;           // Updated content of the message
  optional bool is_pinned = 3;  // Whether the message is pinned
  repeated NotifyEntry notify_entries = 4; // Entries to notify of the update
}
```

**Storage Behavior:**

- Update the content of existing notifications for this `message_id`
- If `is_pinned` is true, mark notification as pinned
- No new notifications are created

### ConsumeMessageDeleted

```
queue: notifications.messages.deleted
exchange name and type: messages.events of type Topic
binding: messages.deleted
```

**Purpose:** When a message is deleted, remove all associated notifications from the storage.

**Message Schema (Proto):**

```protobuf
message DeleteMessageEvent {
  string message_id = 1; // ID of the message being deleted
}
```

**Storage Behavior:**

- Delete all notifications associated with this `message_id`
- Users should no longer see this notification in their inbox

**TODO:** Ensure cleanup of notifications for users who were mentioned but the message was deleted.

### ConsumeFriendsRequestSent

```
queue: notifications.friends.request.sent
exchange name and type: friends.events of type Topic
binding: friends.request.sent
```

**Purpose**: When a friend request is created in the Friend service, create a notification for the recipient.

**Storage Behavior:**

- Create a notification entry for the `recipientId`
- Store reference to `requestId`, `senderId`
- Mark as unread
- Store creation timestamp

**Message Schema:**

```json
{
  "eventId": "evt-fr-001",
  "timestamp": 1733238721000,
  "sourceService": "friends",
  "eventType": "friends.request.sent",
  "data": {
    "requestId": "fr-req-001",
    "senderId": "user-456",
    "recipientId": "user-789",
    "sentAt": 1733238721000
  }
}
```

### ConsumeFriendsRequestAccepted

```
queue: notifications.friends.request.accepted
exchange name and type: friends.events of type Topic
binding: friends.request.accepted
```

**Purpose**: When a friend request is accepted in the Friend service, create a notification for the sender.

**Message Schema:**

```json
{
  "eventId": "evt-fr-001",
  "timestamp": 1733238721000,
  "sourceService": "friends",
  "eventType": "friends.request.accepted",
  "data": {
    "requestId": "fr-req-001",
    "senderId": "user-456",
    "recipientId": "user-789",
    "acceptedAt": 1733238721000
  }
}
```

### ConsumeFriendsRequestCanceled

```
queue: notifications.friends.request.canceled
exchange name and type: friends.events of type Topic
binding: friends.request.canceled
```

**Purpose**: When a friend request is canceled in the Friend service, remove the associated notification from storage.

**Message Schema:**

```json
{
  "eventId": "evt-fr-001",
  "timestamp": 1733238721000,
  "sourceService": "friends",
  "eventType": "friends.request.canceled",
  "data": {
    "requestId": "fr-req-001",
    "senderId": "user-456",
    "recipientId": "user-789",
    "canceledAt": 1733238721000
  }
}
```

**Storage Behavior:**

- Delete the notification associated with this `requestId`
