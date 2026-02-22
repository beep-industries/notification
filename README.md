# Notification Service

This service handles user notifications for various events such as message creation, updates, and deletions, as well as managing user notification preferences. The service interacts with a message broker (RabbitMQ) to receive event messages and update user notifications and settings accordingly.

## Architecture

The service follows Clean Architecture principles with the following layers:

- **API Layer** (`api/`): HTTP handlers and routing
- **Core Layer** (`core/`): Business logic, domain entities, and use cases
- **Infrastructure Layer**: Database and message broker integrations

### Key Components

- **Domain Entities**: `Notification`, `NotificationPreference`, `NotificationId`, `UserId`, `ChannelId`
- **Message Handler**: Processes RabbitMQ messages for create/update/delete events
- **Repository**: Data access layer for notifications and preferences
- **Broker Service**: RabbitMQ consumer integration

## Prerequisites

- Rust toolchain
- Docker and Docker Compose

## Development Setup

### 1. Start Dependencies

```bash
docker compose up -d
```

This starts:

- PostgreSQL database on port 5432
- RabbitMQ with management UI on port 15672
- Keycloak for authentication on port 8080

### 2. Run Database Migrations

```bash
cargo run --bin migrator
```

### 3. Start the Service

```bash
cargo run
```

The service will be available on `http://localhost:3333`.

## Testing

### Unit and Integration Tests

```bash
cargo test
```

Integration tests use testcontainers for RabbitMQ and are automatically cleaned up.
One container per test is used to ensure isolation.

## Configuration

Located in .env file with a `cp .env.example .env` to create your own.

## API Endpoints

### Authentication

All endpoints require a valid JWT token. Get one from Keycloak:

```bash
TOKEN=$(curl -s -X POST "http://localhost:8080/realms/beep/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=client-id" \
  -d "grant_type=password" \
  -d "username=testuser" \
  -d "password=test123" \
  | jq -r .access_token)
```

### Endpoints

#### Health Check

```http
GET /
Authorization: Bearer {token}
```

Example:

```bash
curl -X GET "http://localhost:3333/" \
  -H "Authorization: Bearer $TOKEN"
```

Response:

```json
{
  "message": "Hello, World!",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "testuser"
}
```

#### Get User Notifications

```http
GET /users/{user_id}/notifications
Authorization: Bearer {token}
```

Example:

```bash
curl -X GET "http://localhost:3333/users/bfb1beae-f741-4bc6-9063-b52b8452de91/notifications" \
  -H "Authorization: Bearer $TOKEN"
```

Response:

```json
{
  "notifications": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "user_id": "e715c4e9-6459-4883-9d1a-01dc230bf7cf",
      "title": "New Message",
      "message": "You have a new message",
      "notification_type": "Message",
      "status": "Unread",
      "created_at": "2023-12-31T10:00:00Z"
    }
  ]
}
```

#### Mark Single Notification as Read

```http
PATCH /users/{user_id}/notifications/{notification_id}/read
Authorization: Bearer {token}
```

Example:

```bash
curl -X PATCH "http://localhost:3333/users/bfb1beae-f741-4bc6-9063-b52b8452de91/notifications/223e4567-e89b-12d3-a456-426614174001/read" \
  -H "Authorization: Bearer $TOKEN"
```

Response: `202 Accepted` (empty body)

#### Mark Multiple Notifications as Read

```http
POST /users/{user_id}/notifications/read
Authorization: Bearer {token}
Content-Type: application/json
```

Example:

```bash
curl -X POST "http://localhost:3333/users/bfb1beae-f741-4bc6-9063-b52b8452de91/notifications/read" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "notification_ids": [
      "550e8400-e29b-41d4-a716-446655440000",
      "660e8400-e29b-41d4-a716-446655440001"
    ]
  }'
```

Request body:

```json
{
  "notification_ids": [
    "550e8400-e29b-41d4-a716-446655440000",
    "660e8400-e29b-41d4-a716-446655440001"
  ]
}
```

Response: `202 Accepted` (empty body)

#### Get User Notification Preferences

```http
GET /users/{user_id}/notifications/preferences
Authorization: Bearer {token}
```

Example:

```bash
curl -X GET "http://localhost:3333/users/bfb1beae-f741-4bc6-9063-b52b8452de91/notifications/preferences" \
  -H "Authorization: Bearer $TOKEN"
```

Response:

```json
{
  "notifications_preferences": [
    {
      "id": "456e7890-e89b-12d3-a456-426614174002",
      "user_id": "e715c4e9-6459-4883-9d1a-01dc230bf7cf",
      "channel_id": "789e0123-e89b-12d3-a456-426614174003",
      "enabled": true,
      "muted_until": null
    }
  ]
}
```

#### Update Notification Preferences

```http
PATCH /users/{user_id}/notifications/preferences
Authorization: Bearer {token}
Content-Type: application/json
```

Example:

```bash
curl -X PATCH "http://localhost:3333/users/bfb1beae-f741-4bc6-9063-b52b8452de91/notifications/preferences" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "456e7890-e89b-12d3-a456-426614174002",
    "user_id": "bfb1beae-f741-4bc6-9063-b52b8452de91",
    "channel_id": "789e0123-e89b-12d3-a456-426614174003",
    "enabled": true,
    "muted_until": null
  }'
```

Request body:

```json
{
  "id": "456e7890-e89b-12d3-a456-426614174002",
  "user_id": "e715c4e9-6459-4883-9d1a-01dc230bf7cf",
  "channel_id": "789e0123-e89b-12d3-a456-426614174003",
  "enabled": true,
  "muted_until": null
}
```

Response: `202 Accepted` (empty body)

## Message Broker Integration

The service consumes events from RabbitMQ exchanges:

### Message Created Event

Exchange: `message_created`

```json
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

### Message Updated Event

Exchange: `message_updated`

```json
{
  "message_id": "223e4567-e89b-12d3-a456-426614174001",
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

### Message Deleted Event

Exchange: `message_deleted`

```json
{
  "message_id": "223e4567-e89b-12d3-a456-426614174001"
}
```

## Database Schema

Main tables:

- `notifications`: Stores user notifications
- `notification_preferences`: Stores user notification settings per channel

## Contributing

1. Follow Rust coding standards and run `cargo fmt`
2. Ensure all tests pass: `cargo test`
3. Add tests for new functionality
4. Update this README if adding new features

## Project Governance

[License](https://github.com/beep-industries/.github/blob/main/LICENSE)

[Contributing Guide](https://github.com/beep-industries/.github/blob/main/CONTRIBUTING.md)

[Code of Conduct](https://github.com/beep-industries/.github/blob/main/CODE_OF_CONDUCT.md)
