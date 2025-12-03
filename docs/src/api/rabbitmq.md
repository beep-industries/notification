# RabbitMQ

It is assumed that we are going to use RabbitMQ queues, not streams, to simplify this first iteration. If the need gets identified, we may switch to streams.

## Consumers

This service listens for incoming messages and friends requests in order to send notifications to users.

### ConsumeMessageCreated

```
queue: notifications.messages.created
exchange name and type: messages.events of type Topic
binding: messages.created
```

**Purpose:** When a new message is created in the Messages service, send a notification to the recipient based on their notification settings and DND status.

**Context:** Can be either a DM or a server message.

**Message Schema:**
```
{
    "eventId": "evt-msg-001",
    "timestamp": 1733238721000,
    "sourceService": "messages",
    "eventType": "messages.created",
    "data": {
        "messageId": "msg-123",
        "senderId": "user-456",
        "recipientId": "user-789",
        "content": "Hello, how are you?",
        "createdAt": 1733238721000
    }
}
```

**Notification Behavior:**
- **DM Context:** Check user notification settings + DND status
  - If DND enabled → No notification (silent update to inbox)
  - If notifications disabled for DMs → No notification
  - If notifications enabled → Badge + optional sound/desktop popup
- **Server Context:** Check server/channel settings + DND status
  - If DND enabled → No notification (silent update to inbox)
  - If "All Messages" → Badge + sound + desktop popup
  - If "@Mentions Only" → No notification (just badge)
  - If "Muted" → No notification (just update counter)

### ConsumeMessageUpdated

```
queue: notifications.messages.updated
exchange name and type: messages.events of type Topic
binding: messages.updated
```

**Purpose:** When an existing message is modified, update the message in the user's inbox WITHOUT sending a notification.

**Message Schema:**

```
{
    "eventId": "evt-msg-002",
    "timestamp": 1733238721000,
    "sourceService": "messages",
    "eventType": "messages.updated",
    "data": {
        "messageId": "msg-123",
        "senderId": "user-456",
        "recipientId": "user-789",
        "serverId": null,
        "channelId": null,
        "updatedAt": 1733238721000,
        "updatedFields": ["content", "editedAt"],
        "newContent": "Hello, how are you? (edited)"
    }
}
```

### ConsumeMessageDeleted

```
queue: notifications.messages.deleted
exchange name and type: messages.events of type Topic
binding: messages.deleted
```

**Purpose:** When a message is deleted, update the user's inbox WITHOUT sending a notification.

**Message Schema:**

```
{
    "eventId": "evt-msg-003",
    "timestamp": 1733238721000,
    "sourceService": "messages",
    "eventType": "messages.deleted",
    "data": {
        "messageId": "msg-123",
        "senderId": "user-456",
        "recipientId": "user-789",
        "messageContext": "server",
        "serverId": "srv-001",
        "channelId": "ch-001",
        "deletedAt": 1733238721000
    }
}
```

### ConsumeMessageMentionAdded

```
queue: notifications.messages.mention.added
exchange name and type: messages.events of type Topic
binding: messages.mention.added
```

**Purpose:** When a user is mentioned (@username) in a message, send a HIGH PRIORITY notification to the mentioned user(s), overriding most settings but respecting DND.

**Context:** Can be in a DM or a server message.

**Notification Behavior:**
- **DND Enabled:** NO notification (silent, but mark as unread with mention indicator)
- **DND Disabled:**
  - Desktop popup (highest priority)
  - Sound alert (always plays)
  - Badge with mention highlight
  - Even if channel/server is muted, mention notification plays


**Message Schema:**

{
    "eventId": "evt-msg-mention-001",
    "timestamp": 1733238721000,
    "sourceService": "messages",
    "eventType": "messages.mention.added",
    "data": {
        "messageId": "msg-123",
        "authorId": "user-456",
        "mentionedUserIds": ["user-789", "user-999"],
        "content": "Hey @user-789 and @user-999, check this out!",
        "mentionedAt": 1733238721000
    }
}

### ConsumeMessageMentionRemoved

```
queue: notifications.messages.mention.removed
exchange name and type: messages.events of type Topic
binding: messages.mention.removed
```

**Purpose:** When a mention is removed from a message (message edited or deleted), update the inbox WITHOUT sending a notification.

**Message Schema:**

```
{
    "eventId": "evt-msg-mention-002",
    "timestamp": 1733238721000,
    "sourceService": "messages",
    "eventType": "messages.mention.removed",
    "data": {
        "messageId": "msg-123",
        "authorId": "user-456",
        "messageContext": "server",
        "serverId": "srv-001",
        "previousMentions": [
            {
                "type": "user",
                "userId": "user-789"
            }
        ],
        "removedAt": 1733238721000
    }
}
```

### ConsumeFriendsRequestSent

```
queue: notifications.friends.request.sent
exchange name and type: friends.events of type Topic
binding: friends.request.sent
```

**Purpose**: When a friend request is created in the Friend service, send a notification to the recipient.

**Message Schema:**
```
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
queue: notifications:friends.request.accepted
exchange name and type: friends.events of type Topic
binding: friends.request.accepted
```

**Purpose**: When a friend request is accepted in the Friend service, send a notification to the sender.

***Message Schema:**
```
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
queue: notifications:friends.request.canceled
exchange name and type: friends.events of type Topic
binding: friends.request.canceled
```

**Purpose**: When a friend request is canceled in the Friend service, update the inbox, WITHOUT notifying the user.

***Message Schema:**
```
{
    "eventId": "evt-fr-001",
    "timestamp": 1733238721000,
    "sourceService": "friends",
    "eventType": "friends.request.canceled",
    "data": {
        "requestId": "fr-req-001",
        "senderId": "user-456",
        "recipientId": "user-789",
        "cancelededAt": 1733238721000
    }
}
```