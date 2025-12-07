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