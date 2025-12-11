# Notification Service

```bash
TOKEN=$(curl -s -v \
  -X POST "http://localhost:8080/realms/beep/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=client-id" \
  -d "grant_type=password" \
  -d "username=testuser" \
  -d "password=test123" \
  | jq -r .access_token)
```

```bash
curl -X PATCH "http://localhost:3333/users/e715c4e9-6459-4883-9d1a-01dc230bf7cf/notifications/preferences"   -H "Authorization: Bearer $TOKEN"   -H "Conte
nt-Type: application/json"   -d '{
    "id": "456e7890-e89b-12d3-a456-426614174002",
    "user_id": "e715c4e9-6459-4883-9d1a-01dc230bf7cf",
    "channel_id": "789e0123-e89b-12d3-a456-426614174003",
    "enabled": true,
    "muted_until": null
  }' -v
```

To publish a message in exchange message_created :

```bash

{
  "message_id": "223e4567-e89b-12d3-a456-426614174001",
  "channel_id": "660e8400-e29b-41d4-a716-446655440001",
  "author_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Reply with attachment and mention",
  "reply_to_message_id": "123e4567-e89b-12d3-a456-426614174000",
  "attachments": [
    {
      "id": "333e4567-e89b-12d3-a456-426614174002",
      "name": "screenshot.png",
      "url": "https://example.com/screenshot.png"
    }
  ],
  "notify_entries": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "type": "mention"
    }
  ]
}

```

In message_updated :

```bash

{
  "message_id": "123e4567-e89b-12d3-a456-426614174000",
  "content": "Updated message content!",
  "is_pinned": true,
  "notify_entries": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440003",
      "type": "mention"
    }
  ]
}

```

in message_deleted:

```bash

{
  "message_id": "123e4567-e89b-12d3-a456-426614174000"
}

```
