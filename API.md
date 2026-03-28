# VeriNest Backend API

Base URL:

```text
http://localhost:3000
```

Auth:

- Protected endpoints require `Authorization: Bearer <jwt>`
- Every response includes `x-request-id`
- Optional client headers:
  - `x-request-id: <uuid>`
  - `x-forwarded-for: <client-ip>`
  - `x-real-ip: <client-ip>`

Common error format:

```json
{
  "error": {
    "message": "human readable message"
  }
}
```

Roles:

- `buyer`
- `agent`
- `landlord`

Pagination rules:

- `page` default: `1`
- `per_page` default: `20`
- `per_page` max: `100`

## Health Check

`GET /health`

Success `200`:

```json
{
  "status": "ok",
  "service": "verinest-backend",
  "timestamp": "2026-03-28T08:00:00Z"
}
```

## 1. Register

`POST /auth/register`

Request body:

```json
{
  "full_name": "Buyer One",
  "email": "buyer@example.com",
  "password": "StrongPass123",
  "role": "buyer",
  "phone": "+2348010000000",
  "bio": "Optional profile bio"
}
```

Notes:

- `phone` is optional
- `bio` is optional
- `password` must be at least 8 characters

Success `201`:

```json
{
  "token": "jwt-token",
  "user": {
    "id": "b13fc389-e376-41a3-a31a-e8e4b463ced5",
    "full_name": "Buyer One",
    "email": "buyer@example.com",
    "role": "buyer",
    "bio": "Optional profile bio",
    "created_at": "2026-03-27T12:43:35.984063Z"
  }
}
```

Possible errors:

- `400` invalid input
- `409` user with this email already exists

## 2. Login

`POST /auth/login`

Request body:

```json
{
  "email": "buyer@example.com",
  "password": "StrongPass123"
}
```

Success `200`:

```json
{
  "token": "jwt-token",
  "user": {
    "id": "b13fc389-e376-41a3-a31a-e8e4b463ced5",
    "full_name": "Buyer One",
    "email": "buyer@example.com",
    "role": "buyer",
    "bio": "Buyer",
    "created_at": "2026-03-27T12:43:35.984063Z"
  }
}
```

Possible errors:

- `400` invalid email or missing password
- `401` invalid credentials

## 3. Get User

`GET /users/{id}`

Success `200`:

```json
{
  "id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "full_name": "Agent One",
  "email": "agent@example.com",
  "role": "agent",
  "bio": "Licensed agent",
  "created_at": "2026-03-27T12:43:36.857242Z"
}
```

Possible errors:

- `404` user not found

## 4. List Agents

`GET /agents?page=1&per_page=10`

Query params:

- `page`
- `per_page`

Success `200`:

```json
[
  {
    "id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
    "full_name": "Agent One",
    "email": "agent@example.com",
    "bio": "Licensed agent",
    "operating_city": "Lagos",
    "operating_state": "Lagos",
    "created_at": "2026-03-27T12:43:36.857242Z"
  }
]
```

Possible errors:

- `400` invalid pagination

## 5. Create Property

`POST /properties`

Protected:

- `agent`
- `landlord`

Request body:

```json
{
  "title": "4 Bedroom Duplex",
  "price": 8000000,
  "location": "Ikoyi",
  "exact_address": "5 Banana Island Road, Ikoyi",
  "description": "Owner listed duplex",
  "images": [
    "https://img.example/3.jpg"
  ],
  "contact_name": "Landlord One",
  "contact_phone": "+2348010000002",
  "is_service_apartment": true,
  "agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b"
}
```

Notes:

- `agent_id` is optional
- if the caller is an `agent`, backend sets `agent_id` to the caller automatically
- if the caller is a `landlord`, `agent_id` must belong to an existing agent when provided
- buyers cannot create properties

Success `201`:

```json
{
  "id": "c631ab97-a289-433e-89c3-50a8c182c8de",
  "title": "4 Bedroom Duplex",
  "price": 8000000,
  "location": "Ikoyi",
  "description": "Owner listed duplex",
  "images": [
    "https://img.example/3.jpg"
  ],
  "is_service_apartment": true,
  "owner_id": "8766fabf-af8a-4c8e-80d4-79641a08f3e8",
  "agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "owner_name": "Landlord One",
  "agent_name": "Agent One",
  "exact_address": "5 Banana Island Road, Ikoyi",
  "contact_name": "Landlord One",
  "contact_phone": "+2348010000002",
  "created_at": "2026-03-27T12:43:38.963174Z",
  "updated_at": "2026-03-27T12:43:38.963174Z"
}
```

Possible errors:

- `400` invalid body
- `401` missing or invalid token
- `403` insufficient permissions
- `400` assigned agent does not exist

## 6. List Properties

`GET /properties?page=1&per_page=10&location=Ikoyi&min_price=1000000&max_price=10000000`

Query params:

- `page`
- `per_page`
- `location`
- `min_price`
- `max_price`

Success `200`:

```json
[
  {
    "id": "c631ab97-a289-433e-89c3-50a8c182c8de",
    "title": "4 Bedroom Duplex",
    "price": 8000000,
    "location": "Ikoyi",
    "description": "Owner listed duplex",
    "images": [
      "https://img.example/3.jpg"
    ],
    "is_service_apartment": true,
    "owner_id": "8766fabf-af8a-4c8e-80d4-79641a08f3e8",
    "agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
    "owner_name": "Landlord One",
    "agent_name": "Agent One",
    "created_at": "2026-03-27T12:43:38.963174Z"
  }
]
```

Possible errors:

- `400` invalid pagination

## 7. Get Property Detail

`GET /properties/{id}`

Auth behavior:

- public request: restricted view
- logged in `buyer`: restricted view
- logged in `agent` or `landlord`: full view

Restricted response `200`:

```json
{
  "id": "c631ab97-a289-433e-89c3-50a8c182c8de",
  "title": "4 Bedroom Duplex",
  "price": 8000000,
  "location": "Ikoyi",
  "description": "Owner listed duplex",
  "images": [
    "https://img.example/3.jpg"
  ],
  "is_service_apartment": true,
  "owner_id": "8766fabf-af8a-4c8e-80d4-79641a08f3e8",
  "agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "owner_name": "Landlord One",
  "agent_name": "Agent One",
  "exact_address": null,
  "contact_name": null,
  "contact_phone": null,
  "created_at": "2026-03-27T12:43:38.963174Z",
  "updated_at": "2026-03-27T12:43:38.963174Z"
}
```

Privileged response `200`:

```json
{
  "id": "c631ab97-a289-433e-89c3-50a8c182c8de",
  "title": "4 Bedroom Duplex",
  "price": 8000000,
  "location": "Ikoyi",
  "description": "Owner listed duplex",
  "images": [
    "https://img.example/3.jpg"
  ],
  "is_service_apartment": true,
  "owner_id": "8766fabf-af8a-4c8e-80d4-79641a08f3e8",
  "agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "owner_name": "Landlord One",
  "agent_name": "Agent One",
  "exact_address": "5 Banana Island Road, Ikoyi",
  "contact_name": "Landlord One",
  "contact_phone": "+2348010000002",
  "created_at": "2026-03-27T12:43:38.963174Z",
  "updated_at": "2026-03-27T12:43:38.963174Z"
}
```

Possible errors:

- `404` property not found

## 8. Create Post

`POST /posts`

Protected:

- `buyer`
- `agent`
- `landlord`

Request body:

```json
{
  "budget": 3000000,
  "location": "Lekki",
  "city": "Lagos",
  "state": "Lagos",
  "description": "Looking for a 2 bedroom apartment near Admiralty Way"
}
```

Success `201`:

```json
{
  "id": "d5b71ee7-7374-4393-92ad-d2f49ff2d637"
}
```

Possible errors:

- `400` invalid body
- `401` missing or invalid token

## 9. List Posts

`GET /posts?page=1&per_page=10&location=Lekki&min_budget=1000000&max_budget=4000000`

Query params:

- `page`
- `per_page`
- `location`
- `min_budget`
- `max_budget`

Success `200`:

```json
[
  {
    "id": "d5b71ee7-7374-4393-92ad-d2f49ff2d637",
    "author_id": "b13fc389-e376-41a3-a31a-e8e4b463ced5",
    "author_name": "Buyer One",
    "author_role": "buyer",
    "budget": 3000000,
    "location": "Lekki",
    "city": "Lagos",
    "state": "Lagos",
    "description": "Looking for a 2 bedroom apartment near Admiralty Way",
    "response_count": 0,
    "created_at": "2026-03-27T12:43:39.129864Z"
  }
]
```

Possible errors:

- `400` invalid pagination

## 10. Respond To Post

`POST /posts/{id}/respond`

Protected:

- `buyer`
- `agent`
- `landlord`

Request body:

```json
{
  "message": "I have two matching options in Lekki Phase 1."
}
```

Success `201`:

```json
{
  "id": "87b02982-7ed8-41d8-8c28-ffc4702eafed",
  "post_id": "d5b71ee7-7374-4393-92ad-d2f49ff2d637",
  "responder_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "message": "I have two matching options in Lekki Phase 1.",
  "created_at": "2026-03-27T12:43:39.174098Z"
}
```

## 11. Update Agent Notification Settings

`PATCH /agents/me/notification-settings`

Protected:

- `agent`

Request body:

```json
{
  "notifications_enabled": true,
  "operating_city": "Lagos",
  "operating_state": "Lagos"
}
```

Rules:

- if `notifications_enabled` is `true`, both `operating_city` and `operating_state` are required
- matching is done against new post `city` or `state`

Success `200`:

```json
{
  "notifications_enabled": true,
  "operating_city": "Lagos",
  "operating_state": "Lagos"
}
```

## 12. Agent Post Alerts

`GET /agents/me/post-alerts`

Protected:

- `agent`

Success `200`:

```json
[
  {
    "notification_id": "uuid",
    "post_id": "uuid",
    "author_id": "uuid",
    "author_name": "Buyer One",
    "author_role": "buyer",
    "budget": 3000000,
    "location": "Lekki",
    "city": "Lagos",
    "state": "Lagos",
    "description": "Looking for a 2 bedroom apartment near Admiralty Way",
    "matched_city": "Lagos",
    "matched_state": "Lagos",
    "is_read": false,
    "created_at": "2026-03-28T08:00:00Z"
  }
]
```

## 13. Role Dashboard

`GET /dashboard`

Protected:

- `buyer`
- `agent`
- `landlord`

Buyer success `200`:

```json
{
  "role": "buyer",
  "profile": {
    "id": "uuid",
    "full_name": "Buyer One",
    "email": "buyer@example.com",
    "role": "buyer",
    "bio": "Buyer",
    "created_at": "2026-03-28T08:00:00Z"
  },
  "buyer": {
    "my_posts_count": 2,
    "recent_posts": []
  },
  "agent": null,
  "landlord": null
}
```

Agent success `200`:

```json
{
  "role": "agent",
  "profile": {
    "id": "uuid",
    "full_name": "Agent One",
    "email": "agent@example.com",
    "role": "agent",
    "bio": "Licensed agent",
    "created_at": "2026-03-28T08:00:00Z"
  },
  "buyer": null,
  "agent": {
    "managed_properties_count": 5,
    "service_apartments_count": 2,
    "unread_post_alerts_count": 3,
    "recent_properties": [],
    "recent_post_alerts": []
  },
  "landlord": null
}
```

Landlord success `200`:

```json
{
  "role": "landlord",
  "profile": {
    "id": "uuid",
    "full_name": "Landlord One",
    "email": "landlord@example.com",
    "role": "landlord",
    "bio": "Owner",
    "created_at": "2026-03-28T08:00:00Z"
  },
  "buyer": null,
  "agent": null,
  "landlord": {
    "owned_properties_count": 4,
    "assigned_agents_count": 2,
    "recent_properties": []
  }
}
```

Possible errors:

- `400` missing message
- `401` missing or invalid token
- `404` post not found

## Frontend Field Summary

### Auth user object

```json
{
  "id": "uuid",
  "full_name": "string",
  "email": "string",
  "role": "buyer | agent | landlord",
  "bio": "string | null",
  "created_at": "ISO datetime"
}
```

### Agent list item

```json
{
  "id": "uuid",
  "full_name": "string",
  "email": "string",
  "bio": "string | null",
  "created_at": "ISO datetime"
}
```

### Property list item

```json
{
  "id": "uuid",
  "title": "string",
  "price": 0,
  "location": "string",
  "description": "string",
  "images": ["string"],
  "is_service_apartment": true,
  "owner_id": "uuid",
  "agent_id": "uuid | null",
  "owner_name": "string",
  "agent_name": "string | null",
  "created_at": "ISO datetime"
}
```

### Property detail

```json
{
  "id": "uuid",
  "title": "string",
  "price": 0,
  "location": "string",
  "description": "string",
  "images": ["string"],
  "is_service_apartment": true,
  "owner_id": "uuid",
  "agent_id": "uuid | null",
  "owner_name": "string",
  "agent_name": "string | null",
  "exact_address": "string | null",
  "contact_name": "string | null",
  "contact_phone": "string | null",
  "created_at": "ISO datetime",
  "updated_at": "ISO datetime"
}
```

### Post list item

```json
{
  "id": "uuid",
  "author_id": "uuid",
  "author_name": "string",
  "author_role": "buyer | agent | landlord",
  "budget": 0,
  "location": "string",
  "city": "string",
  "state": "string",
  "description": "string",
  "response_count": 0,
  "created_at": "ISO datetime"
}
```

### Response created

```json
{
  "id": "uuid",
  "post_id": "uuid",
  "responder_id": "uuid",
  "message": "string",
  "created_at": "ISO datetime"
}
```
