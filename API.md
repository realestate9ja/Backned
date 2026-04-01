# VeriNest Backend API

Base URL:

```text
http://localhost:3000
```

Auth:

- Protected endpoints require `Authorization: Bearer <jwt>`
- `POST /admin/bootstrap` requires `x-admin-bootstrap-token`
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
- `admin`

Pagination rules:

- `page` default: `1`
- `per_page` default: `20`
- `per_page` max: `100`

Property lifecycle:

- `draft`
- `pending_verification`
- `verified`
- `published`
- `suspended`

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
  "requested_agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "self_managed": false
}
```

Notes:

- `requested_agent_id` is optional and only meaningful for landlords
- if the caller is an `agent`, backend sets `agent_id` to the caller automatically
- agent-created properties publish immediately
- if the caller is a `landlord` with no requested agent, the property is created as `self_managed: true`
- landlord properties start as `pending_verification`
- if a landlord requests an agent, the backend stores a `property_agent_request` and agent assignment must happen after verification
- the `images` array can contain image URLs and service-apartment video URLs for MVP
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
  "status": "pending_verification",
  "self_managed": false,
  "owner_id": "8766fabf-af8a-4c8e-80d4-79641a08f3e8",
  "agent_id": null,
  "owner_name": "Landlord One",
  "agent_name": null,
  "exact_address": "5 Banana Island Road, Ikoyi",
  "contact_name": "Landlord One",
  "contact_phone": "+2348010000002",
  "verified_by": null,
  "verified_at": null,
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

Only `published` properties appear in public property listing endpoints.

## Workflow Endpoints

### Verify Property

`POST /properties/{id}/verify`

Protected:

- `agent`

Moves a property from `pending_verification` to `verified`.

### Publish Property

`POST /properties/{id}/publish`

Protected:

- property owner
- assigned agent

Moves a property from `verified` to `published`.

### Request Agent For Property

`POST /properties/{id}/agent-request`

Protected:

- `landlord`

Request body:

```json
{
  "requested_agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "notes": "Need an agent for inspection and tenant coordination"
}
```

### Assign Agent To Property

`POST /properties/{id}/assign-agent`

Protected:

- `landlord`

Request body:

```json
{
  "agent_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b"
}
```

Property must already be `verified`.

### Create Request Thread Message

`POST /responses/{id}/thread/messages`

Protected:

- response buyer
- response responder

Request body:

```json
{
  "message": "Can you show a live walkthrough tonight?"
}
```

### Get Request Thread

`GET /responses/{id}/thread`

Protected:

- response buyer
- response responder

### Create Live Video Session

`POST /responses/{id}/live-video-sessions`

Protected:

- response buyer

Request body:

```json
{
  "scheduled_at": "2026-03-30T18:30:00Z",
  "tracking_notes": "Buyer requested a WhatsApp walkthrough"
}
```

Notes:

- video is not stored
- `recording_saved` is always `false`
- provider is `livekit`
- session lifecycle uses `requested`, `scheduled`, `live`, `completed`, `cancelled`

### Update Live Video Session

`PATCH /live-video-sessions/{id}`

Protected:

- session buyer
- session agent/responder

Request body:

```json
{
  "status": "completed",
  "started_at": "2026-03-30T18:30:00Z",
  "ended_at": "2026-03-30T18:47:00Z",
  "tracking_notes": "Buyer requested a second balcony sweep"
}
```

### Get Live Video Session Access

`GET /live-video-sessions/{id}`

Protected:

- session buyer
- session agent/responder

Success `200`:

```json
{
  "session": {
    "id": "80b8fa5c-1474-40b1-9e0a-ffef28166461",
    "provider": "livekit",
    "room_name": "verinest-live-80b8fa5c-1474-40b1-9e0a-ffef28166461",
    "status": "requested"
  },
  "server_url": "wss://your-livekit-host",
  "room_name": "verinest-live-80b8fa5c-1474-40b1-9e0a-ffef28166461",
  "participant_identity": "buyer:9e46700d-3ce1-4a07-b362-4a1fd6375564",
  "participant_name": "Buyer One",
  "token": "livekit-join-jwt"
}
```

### Create Site Visit

`POST /responses/{id}/site-visits`

Protected:

- response buyer

Request body:

```json
{
  "property_id": "c631ab97-a289-433e-89c3-50a8c182c8de",
  "scheduled_at": "2026-03-31T10:00:00Z",
  "meeting_point": "Main gate, Wuse 2"
}
```

### Update Site Visit

`PATCH /site-visits/{id}`

Request body:

```json
{
  "status": "completed",
  "meeting_point": "Reception lobby"
}
```

### Certify Site Visit

`POST /site-visits/{id}/certify`

Request body:

```json
{
  "notes": "Visit completed successfully and property condition matched listing"
}
```

### Reviews

`POST /reviews`

Protected:

- authenticated users

Request body:

```json
{
  "reviewee_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "property_id": "c631ab97-a289-433e-89c3-50a8c182c8de",
  "response_id": "d0e2969c-a341-45b1-b78f-fd4d833f2f80",
  "rating": 5,
  "comment": "Responsive agent and accurate listing"
}
```

`GET /users/{id}/reviews`

### Reports

`POST /reports`

Protected:

- authenticated users

Request body:

```json
{
  "reported_user_id": "fb02bbf8-de20-495f-b676-4d3750bc5f8b",
  "property_id": "c631ab97-a289-433e-89c3-50a8c182c8de",
  "response_id": "d0e2969c-a341-45b1-b78f-fd4d833f2f80",
  "violation_type": "fraud",
  "reason": "misleading_listing",
  "details": "The exact apartment shown on video was different from the photos"
}
```

Violation types:

- `quality`
- `fraud`
- `other`

### Admin Moderation

`POST /admin/reports/{id}/decision`

Protected:

- `admin`

Request body:

```json
{
  "status": "upheld",
  "review_notes": "Evidence confirmed. Property suspended."
}
```

Moderation statuses:

- `upheld`
- `dismissed`

Enforcement:

- property gets suspended after 3 low-rated reviews (`rating <= 2`)
- upheld quality reports increase quality strikes
- upheld fraud reports can suspend the property immediately
- repeated fraud strikes can ban the reported account

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
  "request_title": "Need serviced apartment in Wuse 2",
  "area": "Wuse 2",
  "city": "Abuja",
  "state": "FCT",
  "property_type": "service_apartment",
  "bedrooms": 2,
  "min_budget": 3500000,
  "max_budget": 5000000,
  "pricing_preference": "monthly",
  "desired_features": [
    "gym",
    "24/7 power",
    "parking"
  ],
  "description": "Looking for a serviced apartment close to business district"
}
```

Notes:

- `desired_features` must not be empty
- `max_budget` must be greater than or equal to `min_budget`
- `bedrooms` must be `0` or greater

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

`GET /posts?page=1&per_page=10&property_type=service_apartment&city=Abuja&state=FCT&min_budget=3000000&max_budget=6000000`

Query params:

- `page`
- `per_page`
- `location`
- `property_type`
- `city`
- `state`
- `min_budget`
- `max_budget`

Success `200`:

```json
[
  {
    "id": "66f0008d-dfc7-4c9d-82b1-bedcb959c57a",
    "author_id": "efbf841d-373f-4853-b950-7c3c00db6b54",
    "author_name": "Buyer Three",
    "author_role": "buyer",
    "location": "Wuse 2, Abuja",
    "request_title": "Need serviced apartment in Wuse 2",
    "area": "Wuse 2",
    "city": "Abuja",
    "state": "FCT",
    "property_type": "service_apartment",
    "bedrooms": 2,
    "min_budget": 3500000,
    "max_budget": 5000000,
    "pricing_preference": "monthly",
    "desired_features": [
      "gym",
      "24/7 power",
      "parking"
    ],
    "status": "active",
    "description": "Looking for a serviced apartment close to business district",
    "response_count": 1,
    "created_at": "2026-03-28T12:39:20.370277Z"
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
  "message": "I have a serviced apartment that matches your request.",
  "property_ids": [
    "c34a4043-fafc-4527-ad8c-623b13e6f80a"
  ]
}
```

Rules:

- `property_ids` may be empty
- when `property_ids` are provided, the responder must own or manage those properties on the platform

Success `201`:

```json
{
  "id": "38d4554e-76aa-4329-8ede-48d6e3523225",
  "post_id": "66f0008d-dfc7-4c9d-82b1-bedcb959c57a",
  "responder_id": "11f32bdb-4b5a-423e-8bea-f8704b99c862",
  "message": "I have a serviced apartment that matches your request.",
  "properties": [
    {
      "id": "c34a4043-fafc-4527-ad8c-623b13e6f80a",
      "title": "Serviced 2BR Apartment",
      "price": 4200000,
      "location": "Wuse 2, Abuja",
      "description": "Serviced apartment with gym and power",
      "images": [
        "https://img.example/p1.jpg"
      ],
      "is_service_apartment": true,
      "owner_id": "11f32bdb-4b5a-423e-8bea-f8704b99c862",
      "agent_id": "11f32bdb-4b5a-423e-8bea-f8704b99c862",
      "owner_name": "Agent Three",
      "agent_name": "Agent Three",
      "created_at": "2026-03-28T12:39:20.235733Z"
    }
  ],
  "created_at": "2026-03-28T12:39:21.017164Z"
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
    "location": "Wuse 2, Abuja",
    "request_title": "Need serviced apartment in Wuse 2",
    "area": "Wuse 2",
    "city": "Abuja",
    "state": "FCT",
    "property_type": "service_apartment",
    "bedrooms": 2,
    "min_budget": 3500000,
    "max_budget": 5000000,
    "pricing_preference": "monthly",
    "desired_features": ["gym", "24/7 power", "parking"],
    "status": "active",
    "description": "Looking for a serviced apartment close to business district",
    "matched_city": "Abuja",
    "matched_state": "FCT",
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
    "active_requests": [
      {
        "request": {
          "id": "uuid",
          "author_id": "uuid",
          "author_name": "Buyer One",
          "author_role": "buyer",
          "location": "Wuse 2, Abuja",
          "request_title": "Need serviced apartment in Wuse 2",
          "area": "Wuse 2",
          "city": "Abuja",
          "state": "FCT",
          "property_type": "service_apartment",
          "bedrooms": 2,
          "min_budget": 3500000,
          "max_budget": 5000000,
          "pricing_preference": "monthly",
          "desired_features": ["gym", "24/7 power", "parking"],
          "status": "active",
          "description": "Looking for a serviced apartment",
          "response_count": 1,
          "created_at": "2026-03-28T08:00:00Z"
        },
        "responses": [
          {
            "response_id": "uuid",
            "responder_id": "uuid",
            "responder_name": "Agent One",
            "responder_role": "agent",
            "message": "I have a serviced apartment that matches your request.",
            "properties": [],
            "created_at": "2026-03-28T08:00:00Z"
          }
        ]
      }
    ]
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
    "managed_properties": [],
    "service_apartments": [],
    "unread_post_alerts": []
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
    "owned_properties": []
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
  "location": "string",
  "request_title": "string",
  "area": "string",
  "city": "string",
  "state": "string",
  "property_type": "string",
  "bedrooms": 0,
  "min_budget": 0,
  "max_budget": 0,
  "pricing_preference": "string",
  "desired_features": ["string"],
  "status": "active | closed | fulfilled",
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
  "properties": [
    {
      "id": "uuid"
    }
  ],
  "created_at": "ISO datetime"
}
```
