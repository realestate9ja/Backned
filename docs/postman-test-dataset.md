# Postman Test Dataset

Base URL used in the verified run:

```text
http://127.0.0.1:3010
```

Internal moderation header used in the verified run:

```text
x-moderation-key: test-moderation-key
```

## Accounts

Use this password for every account:

```text
StrongPass123
```

Accounts:

- `buyer1@example.com`
- `buyer2@example.com`
- `buyer3@example.com`
- `agent1@example.com`
- `agent2@example.com`
- `landlord1@example.com`

## IDs From The Successful Test Run

```json
{
  "buyer1_id": "9e46700d-3ce1-4a07-b362-4a1fd6375564",
  "buyer2_id": "7c0f97fe-ca1d-49a1-8ec8-86847b0e8991",
  "buyer3_id": "aa742d7b-720d-47b4-8f4c-5c1bd908935e",
  "agent1_id": "82a0871b-ff2b-483f-98cf-a47148d8161b",
  "agent2_id": "649a589d-36a4-4ce9-b820-d54e81ce0779",
  "landlord1_id": "6514c4bd-b603-43c8-b035-88f518f250cd",
  "agent1_property_id": "fa077f65-ce4b-420c-99f8-6b0bdc0bcb39",
  "agent2_property_id": "c2d2a68f-b17d-4dae-b0ed-88cf988ca618",
  "landlord_self_id": "24578f8d-f6fd-46bb-a422-05ed2c4e7f55",
  "landlord_agent_id": "b8f3a3e3-db66-4fd9-bd26-1d7a1936db23",
  "post_id": "0ff02cb3-76c0-406c-92a2-30f76ec4cf8f",
  "response_id": "5ee82ae7-3c31-4406-9d28-736f4aae8b2c",
  "video_id": "80b8fa5c-1474-40b1-9e0a-ffef28166461",
  "site_visit_id": "8703a326-6599-4c39-8f24-9f8740824d53",
  "report1_id": "a216a882-7c9a-414e-a11d-d3be51fef6cd",
  "report2_id": "e1f5fc01-7ae2-402e-9945-ec76c26543de",
  "report3_id": "5455d250-267a-415c-8932-9ed8b4d7e250"
}
```

## Request Bodies

### Register Buyer 1

```json
{
  "full_name": "Buyer One",
  "email": "buyer1@example.com",
  "password": "StrongPass123",
  "role": "buyer",
  "bio": "Buyer 1"
}
```

### Register Buyer 2

```json
{
  "full_name": "Buyer Two",
  "email": "buyer2@example.com",
  "password": "StrongPass123",
  "role": "buyer",
  "bio": "Buyer 2"
}
```

### Register Buyer 3

```json
{
  "full_name": "Buyer Three",
  "email": "buyer3@example.com",
  "password": "StrongPass123",
  "role": "buyer",
  "bio": "Buyer 3"
}
```

### Register Agent 1

```json
{
  "full_name": "Agent Prime",
  "email": "agent1@example.com",
  "password": "StrongPass123",
  "role": "agent",
  "phone": "+2348010000101",
  "bio": "Main agent"
}
```

### Register Agent 2

```json
{
  "full_name": "Agent Risk",
  "email": "agent2@example.com",
  "password": "StrongPass123",
  "role": "agent",
  "phone": "+2348010000102",
  "bio": "Secondary agent"
}
```

### Register Landlord

```json
{
  "full_name": "Landlord One",
  "email": "landlord1@example.com",
  "password": "StrongPass123",
  "role": "landlord",
  "phone": "+2348010000201",
  "bio": "Owner"
}
```

### Agent Notification Settings

```json
{
  "notifications_enabled": true,
  "operating_city": "Abuja",
  "operating_state": "FCT"
}
```

### Agent 1 Published Property

```json
{
  "title": "Sunrise Suites",
  "price": 4200000,
  "location": "Wuse 2, Abuja",
  "exact_address": "14 Adetokunbo Ademola Crescent, Wuse 2, Abuja",
  "description": "Serviced apartment with mixed media",
  "images": [
    "https://cdn.example.com/sunrise-front.jpg",
    "https://cdn.example.com/sunrise-tour.mp4"
  ],
  "contact_name": "Agent Prime",
  "contact_phone": "+2348010000101",
  "is_service_apartment": true
}
```

### Agent 2 Property

```json
{
  "title": "River View Apartment",
  "price": 3800000,
  "location": "Gwarinpa, Abuja",
  "exact_address": "7 Crescent, Gwarinpa, Abuja",
  "description": "Potentially problematic listing for moderation tests",
  "images": [
    "https://cdn.example.com/river-view.jpg"
  ],
  "contact_name": "Agent Risk",
  "contact_phone": "+2348010000102",
  "is_service_apartment": false
}
```

### Landlord Self-Managed Property

```json
{
  "title": "Oak Residence",
  "price": 5100000,
  "location": "Maitama, Abuja",
  "exact_address": "22 Nile Street, Maitama, Abuja",
  "description": "Landlord self-managed property",
  "images": [
    "https://cdn.example.com/oak-residence.jpg"
  ],
  "contact_name": "Landlord One",
  "contact_phone": "+2348010000201",
  "is_service_apartment": false,
  "self_managed": true
}
```

### Landlord Property With Agent Request

```json
{
  "title": "Central Heights",
  "price": 6100000,
  "location": "Asokoro, Abuja",
  "exact_address": "3 Yakubu Gowon Crescent, Asokoro, Abuja",
  "description": "Landlord property requiring agent",
  "images": [
    "https://cdn.example.com/central-heights.jpg"
  ],
  "contact_name": "Landlord One",
  "contact_phone": "+2348010000201",
  "is_service_apartment": false,
  "self_managed": false,
  "requested_agent_id": "82a0871b-ff2b-483f-98cf-a47148d8161b"
}
```

### Buyer Request Post

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
    "24/7 power",
    "parking",
    "wifi"
  ],
  "description": "Looking for a clean serviced apartment near business district"
}
```

### Post Response

```json
{
  "message": "I have two matching properties for you",
  "property_ids": [
    "fa077f65-ce4b-420c-99f8-6b0bdc0bcb39",
    "b8f3a3e3-db66-4fd9-bd26-1d7a1936db23"
  ]
}
```

### Thread Message

```json
{
  "message": "Can you do a live walkthrough this evening?"
}
```

### Live Video Session

```json
{
  "scheduled_at": "2026-03-31T18:30:00Z",
  "tracking_notes": "Buyer requested a same-day walkthrough"
}
```

### Live Video Update

```json
{
  "status": "completed",
  "started_at": "2026-03-31T18:30:00Z",
  "ended_at": "2026-03-31T18:48:00Z",
  "tracking_notes": "Walkthrough completed, buyer requested in-person visit"
}
```

### Site Visit

```json
{
  "property_id": "fa077f65-ce4b-420c-99f8-6b0bdc0bcb39",
  "scheduled_at": "2026-04-01T10:00:00Z",
  "meeting_point": "Sunrise Suites front gate"
}
```

### Site Visit Update

```json
{
  "status": "completed",
  "meeting_point": "Reception desk"
}
```

### Site Visit Certification

```json
{
  "notes": "Visited in person and property matched the listing"
}
```

### Positive Review

```json
{
  "reviewee_id": "82a0871b-ff2b-483f-98cf-a47148d8161b",
  "property_id": "fa077f65-ce4b-420c-99f8-6b0bdc0bcb39",
  "response_id": "5ee82ae7-3c31-4406-9d28-736f4aae8b2c",
  "rating": 5,
  "comment": "Helpful agent and accurate walkthrough"
}
```

### Bad Review Template

```json
{
  "reviewee_id": "649a589d-36a4-4ce9-b820-d54e81ce0779",
  "property_id": "c2d2a68f-b17d-4dae-b0ed-88cf988ca618",
  "rating": 1,
  "comment": "Listing details were inaccurate"
}
```

### Fraud Report Template

```json
{
  "reported_user_id": "649a589d-36a4-4ce9-b820-d54e81ce0779",
  "property_id": "c2d2a68f-b17d-4dae-b0ed-88cf988ca618",
  "violation_type": "fraud",
  "reason": "bait_and_switch",
  "details": "The listing looked fake after inspection"
}
```

### Internal Moderation Decision

```json
{
  "status": "upheld",
  "review_notes": "Fraud evidence confirmed"
}
```

## Recommended Postman Run Order

1. `GET /health`
2. Register all six accounts
3. Login and save tokens
4. `GET /users/{agent1_id}`
5. `GET /agents`
6. `PATCH /agents/me/notification-settings`
7. Create the four properties
8. Verify, assign, and publish the landlord-agent property
9. Create the buyer request post
10. Respond to the post
11. Create thread message and fetch thread
12. Create and update live video session
13. Create, update, and certify site visit
14. Create positive review and fetch user reviews
15. Submit three bad reviews for `agent2_property_id`
16. Submit and uphold three fraud reports for `agent2_id`
17. Confirm banned agent can no longer create properties or log in
18. Fetch buyer, agent, and landlord dashboards
