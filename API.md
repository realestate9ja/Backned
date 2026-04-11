# VeriNest API

Base URL:

```text
http://localhost:3000
```

Primary compatibility prefix:

```text
/api/v1
```

Auth:

- Protected endpoints require `Authorization: Bearer <jwt>`
- `POST /admin/bootstrap` requires `x-admin-bootstrap-token`
- Login and register return both `token` and `refresh_token`
- `GET /auth/verify-email` and `GET /api/v1/auth/verify-email` share the same behavior

Roles:

- `seeker`
- `agent`
- `landlord`
- `admin`

Common error format:

```json
{
  "error": {
    "message": "human readable message"
  }
}
```

## Auth

### `POST /api/v1/auth/register`

Request:

```json
{
  "full_name": "Seeker One",
  "email": "seeker@example.com",
  "password": "StrongPass123",
  "phone": "+2348010000000",
  "bio": "Optional bio"
}
```

Success `201`:

```json
{
  "token": "jwt-token",
  "refresh_token": "refresh-token",
  "user": {
    "id": "e1c6d38e-7b85-4d18-b0eb-090d6d742c8e",
    "full_name": "Seeker One",
    "email": "seeker@example.com",
    "email_verified": false,
    "role": "unassigned",
    "bio": "Optional bio",
    "average_rating": null,
    "review_count": 0,
    "verification_status": "not_required",
    "created_at": "2026-04-06T09:00:00Z"
  }
}
```

Notes:

- `/api/v1/auth/register` creates an `unassigned` account
- role assignment happens at `POST /api/v1/onboarding/role`

### `POST /api/v1/auth/login`

Request:

```json
{
  "email": "seeker@example.com",
  "password": "StrongPass123"
}
```

Success `200`: same shape as register.

Notes:

- if `email_verified` is `false`, login sends a verification email using the configured mail provider

### `GET /api/v1/auth/verify-email?token={token}`

Success `200`:

```json
{
  "id": "e1c6d38e-7b85-4d18-b0eb-090d6d742c8e",
  "full_name": "Seeker One",
  "email": "seeker@example.com",
  "email_verified": true,
  "role": "seeker",
  "bio": "Optional bio",
  "average_rating": null,
  "review_count": 0,
  "verification_status": "not_required",
  "created_at": "2026-04-06T09:00:00Z"
}
```

### `POST /api/v1/auth/send-email-code`

Request:

```json
{
  "email": "seeker@example.com",
  "purpose": "signup"
}
```

Success `200`:

```json
{
  "ok": true,
  "expires_in_seconds": 600,
  "code_length": 6
}
```

### `POST /api/v1/auth/verify-email-code`

Request:

```json
{
  "email": "seeker@example.com",
  "code": "123456"
}
```

Success `200`: returns the verified user shape.

### `POST /api/v1/auth/refresh`

Request:

```json
{
  "refreshToken": "refresh-token"
}
```

Success `200`: same shape as register, with a new `refresh_token`.

### `POST /api/v1/auth/logout`

Request:

```json
{
  "refreshToken": "refresh-token"
}
```

Success `204`

### `GET /api/v1/auth/me`

Success `200`:

```json
{
  "user": {
    "id": "e1c6d38e-7b85-4d18-b0eb-090d6d742c8e",
    "full_name": "Seeker One",
    "email": "seeker@example.com",
    "email_verified": false,
    "role": "seeker",
    "bio": "Optional bio",
    "average_rating": null,
    "review_count": 0,
    "verification_status": "not_required",
    "created_at": "2026-04-06T09:00:00Z"
  },
  "profile": {
    "id": "e1c6d38e-7b85-4d18-b0eb-090d6d742c8e",
    "userId": "e1c6d38e-7b85-4d18-b0eb-090d6d742c8e",
    "fullName": "Seeker One",
    "phone": null,
    "city": null,
    "avatarUrl": null,
    "bio": "Optional bio",
    "onboardingCompleted": false,
    "createdAt": "2026-04-06T09:00:00Z",
    "updatedAt": "2026-04-06T09:00:00Z"
  },
  "roleProfile": null,
  "verification": null
}
```

## Onboarding

### `POST /api/v1/onboarding/role`

Protected.

Request:

```json
{
  "role": "seeker"
}
```

Allowed values:

- `seeker`
- `agent`
- `landlord`

Notes:

- only available while the user role is `unassigned`
- role can only be selected once

### `PUT /api/v1/onboarding/profile`

Protected.

Seeker request:

```json
{
  "role": "seeker",
  "phone": "+2348012345678",
  "city": "Lagos",
  "preferredCity": "Lagos",
  "preferredAccommodationType": "Rent",
  "preferredBudgetLabel": "500k-1m",
  "moveInTimeline": "Within 1 month"
}
```

Agent request:

```json
{
  "role": "agent",
  "phone": "+2348012345678",
  "city": "Lagos",
  "companyName": "Prime Realtors Ltd",
  "experienceRange": "3-5 years",
  "specializations": ["Residential", "Luxury"],
  "bio": "Focused on premium residential leasing."
}
```

Landlord request:

```json
{
  "role": "landlord",
  "phone": "+2348012345678",
  "city": "Abuja",
  "propertyCountRange": "2-5",
  "propertyTypes": ["Flat / Apartment", "Duplex"],
  "currentAgentStatus": "No"
}
```

Success `200`: same shape as `GET /api/v1/auth/me`, with populated `roleProfile`.

## Verifications

### `POST /api/v1/verifications`

Protected: `agent`, `landlord`

Request:

```json
{
  "notes": "Agent KYC submitted"
}
```

Success `201`:

```json
{
  "id": "df9b1e9c-4ef8-47f1-8d77-c9ac75e520a8",
  "userId": "0bf4f4d5-48a9-4b7b-aaf4-77e805fc2902",
  "status": "submitted",
  "submittedAt": "2026-04-06T09:05:00Z",
  "reviewedAt": null,
  "reviewedBy": null,
  "rejectionReason": null,
  "notes": "Agent KYC submitted",
  "createdAt": "2026-04-06T09:05:00Z",
  "updatedAt": "2026-04-06T09:05:00Z"
}
```

### `POST /api/v1/verifications/{id}/documents`

Request:

```json
{
  "documentType": "nin",
  "fileUrl": "https://cdn.example.com/agent-nin.pdf",
  "fileKey": "agent-nin.pdf",
  "mimeType": "application/pdf"
}
```

### `GET /api/v1/verifications/me`

Success `200`:

```json
{
  "verification": {
    "id": "df9b1e9c-4ef8-47f1-8d77-c9ac75e520a8",
    "userId": "0bf4f4d5-48a9-4b7b-aaf4-77e805fc2902",
    "status": "submitted",
    "submittedAt": "2026-04-06T09:05:00Z",
    "reviewedAt": null,
    "reviewedBy": null,
    "rejectionReason": null,
    "notes": "Agent KYC submitted",
    "createdAt": "2026-04-06T09:05:00Z",
    "updatedAt": "2026-04-06T09:05:00Z"
  },
  "documents": [
    {
      "id": "fb3ab324-240d-4c46-98b8-65c7b02d2c46",
      "verificationId": "df9b1e9c-4ef8-47f1-8d77-c9ac75e520a8",
      "documentType": "nin",
      "fileUrl": "https://cdn.example.com/agent-nin.pdf",
      "fileKey": "agent-nin.pdf",
      "mimeType": "application/pdf",
      "status": "uploaded",
      "createdAt": "2026-04-06T09:06:00Z"
    }
  ]
}
```

## Public Properties

### `GET /api/v1/properties`

Uses the existing public property feed.

Supported query params:

- `page`
- `per_page`
- `location`
- `min_price`
- `max_price`

### `GET /api/v1/properties/{id}`

Protected and public-safe.

For seekers and unauthenticated callers, sensitive fields are hidden:

- `exact_address`
- `contact_name`
- `contact_phone`

## Agent Properties

### `POST /api/v1/agent/properties`

Protected: `agent`

Request:

```json
{
  "title": "Ocean View Apartment",
  "price": 4500000,
  "location": "Lekki Phase 1",
  "exact_address": "10 Admiralty Way, Lekki",
  "description": "Serviced apartment with video tour",
  "images": [
    "https://cdn.example.com/front.jpg",
    "https://cdn.example.com/tour.mp4"
  ],
  "contact_name": "Agent One",
  "contact_phone": "+2348010000001",
  "is_service_apartment": true
}
```

Notes:

- agent must be verified before listing
- media is currently stored in `images` as URL strings, including video URLs

### `GET /api/v1/agent/properties`

Protected: `agent`

Returns the agent’s owned or managed properties.

### `PATCH /api/v1/agent/properties/{id}`

Protected: `agent`

Updates an owned or managed property.

## Seeker Needs

### `POST /api/v1/seeker/needs`

Protected: `seeker`

Request:

```json
{
  "request_title": "Need 2-bed serviced apartment",
  "area": "Lekki Phase 1",
  "city": "Lagos",
  "state": "Lagos",
  "property_type": "service_apartment",
  "bedrooms": 2,
  "min_budget": 3000000,
  "max_budget": 5000000,
  "pricing_preference": "monthly",
  "desired_features": ["wifi", "parking"],
  "description": "Looking for furnished apartment"
}
```

Success `201`:

```json
{
  "id": "3cfe8fb0-96cb-4c8a-aa9a-8a9e8a39a46f"
}
```

### `GET /api/v1/seeker/needs`

Protected: `seeker`

Returns the seeker’s own need posts.

## Agent Leads

### `GET /api/v1/agent/leads`

Protected: `agent`

Returns agent lead rows derived from post notifications and optional `lead_matches`.

Success item shape:

```json
{
  "id": "0b842cfd-c4d5-48b6-80b6-93d9cc1d8bd5",
  "needPostId": "3cfe8fb0-96cb-4c8a-aa9a-8a9e8a39a46f",
  "matchedPropertyId": null,
  "matchScore": 0,
  "status": "new",
  "slaExpiresAt": null,
  "createdAt": "2026-04-06T09:12:00Z",
  "updatedAt": "2026-04-06T09:12:00Z",
  "requestTitle": "Need 2-bed serviced apartment",
  "location": "Lekki Phase 1, Lagos",
  "propertyType": "service_apartment",
  "urgency": null
}
```

### `GET /api/v1/agent/leads/{id}`

Protected: `agent`

Returns:

- `lead`
- `seekerNeed`
- `matchedProperties`
- `existingOffer`

## Offers

### `POST /api/v1/offers`

Protected: `agent`, `landlord`

Request:

```json
{
  "needPostId": "3cfe8fb0-96cb-4c8a-aa9a-8a9e8a39a46f",
  "propertyId": "4d7d50b8-5a4b-4abf-80e7-587855edcff0",
  "offerPriceAmount": 4500000,
  "offerPriceCurrency": "NGN",
  "offerPricePeriod": "year",
  "message": "This matches your request",
  "prioritySend": true
}
```

### `GET /api/v1/seeker/offers`

Protected: `seeker`

Returns offers against the seeker’s needs.

## Saved Properties

### `POST /api/v1/seeker/saved-properties`

Protected: `seeker`

Request:

```json
{
  "propertyId": "4d7d50b8-5a4b-4abf-80e7-587855edcff0"
}
```

### `GET /api/v1/seeker/saved-properties`

Protected: `seeker`

### `DELETE /api/v1/seeker/saved-properties/{propertyId}`

Protected: `seeker`

Success `204`

## Bookings

### `POST /api/v1/bookings`

Protected: `seeker`

Request:

```json
{
  "offerId": "c2962262-d3f0-45d0-876d-2db9f454f5d2",
  "propertyId": "4d7d50b8-5a4b-4abf-80e7-587855edcff0",
  "bookingType": "viewing",
  "scheduledFor": "2026-04-10T10:00:00Z",
  "notes": "Please confirm"
}
```

### `GET /api/v1/seeker/bookings`

Protected: `seeker`

### `GET /api/v1/agent/bookings`

Protected: `agent`

### `GET /api/v1/agent/payouts`

Protected: `agent`

### `GET /api/v1/agent/calendar`

Protected: `agent`

### `GET /api/v1/seeker/dashboard/overview`

Protected: `seeker`

Returns:

- `stats`
- `matchTrends`
- `savedProperties`
- `recentOffers`

### `GET /api/v1/agent/dashboard/overview`

Protected: `agent`

Returns:

- `stats`
- `earningsSeries`
- `topListings`
- `recentLeads`

## Landlord

### `GET /api/v1/landlord/dashboard/overview`

Protected: `landlord`

Returns:

- `stats`
- `occupancySeries`
- `collectionSeries`
- `leaseExpiries`
- `maintenanceQueue`

### `POST /api/v1/landlord/properties`

Protected: `landlord`

### `GET /api/v1/landlord/properties`

Protected: `landlord`

### `POST /api/v1/landlord/units`

Protected: `landlord`

### `GET /api/v1/landlord/units`

Protected: `landlord`

Unit item shape:

```json
{
  "id": "11111111-1111-1111-1111-111111111111",
  "propertyId": "03e8c35e-f352-4771-a155-7a9a7a5615f5",
  "unitCode": "A1",
  "name": "Palm Residence A1",
  "unitType": "flat",
  "bedroomsLabel": "2-bed",
  "rentAmount": 850000,
  "rentCurrency": "NGN",
  "rentPeriod": "year",
  "occupancyStatus": "occupied",
  "listingStatus": "listed",
  "tenantUserId": "e1c6d38e-7b85-4d18-b0eb-090d6d742c8e",
  "leaseId": "22222222-2222-2222-2222-222222222222",
  "createdAt": "2026-04-06T09:20:00Z",
  "updatedAt": "2026-04-06T09:20:00Z"
}
```

### `GET /api/v1/landlord/collections`

Protected: `landlord`

### `GET /api/v1/landlord/payouts`

Protected: `landlord`

### `GET /api/v1/landlord/maintenance`

Protected: `landlord`

### `POST /api/v1/landlord/maintenance`

Protected: `landlord`

### `GET /api/v1/landlord/calendar`

Protected: `landlord`

Calendar item shape:

```json
{
  "id": "77777777-7777-7777-7777-777777777777",
  "userId": "b51dfdb8-0fd9-4438-a4ae-1d69ac6e2ab8",
  "propertyId": "03e8c35e-f352-4771-a155-7a9a7a5615f5",
  "unitId": "11111111-1111-1111-1111-111111111111",
  "eventType": "rent_followup",
  "title": "Palm Residence A1",
  "startsAt": "2026-04-08T09:00:00Z",
  "endsAt": "2026-04-08T10:00:00Z",
  "status": "scheduled",
  "metadataJson": {
    "label": "N850,000"
  },
  "createdAt": "2026-04-06T09:22:00Z"
}
```

## Admin

### `GET /api/v1/admin/metrics/overview`

Protected: `admin`

Success `200`:

```json
{
  "totalProperties": 2,
  "activeUsers": 4,
  "monthlyRevenue": 850000,
  "openDisputes": 1
}
```

### `GET /api/v1/admin/users`

Protected: `admin`

### `GET /api/v1/admin/properties`

Protected: `admin`

### `GET /api/v1/admin/transactions`

Protected: `admin`

### `GET /api/v1/admin/disputes`

Protected: `admin`

### `GET /api/v1/admin/reports`

Protected: `admin`

### `GET /api/v1/admin/announcements`

Protected: `admin`

### `POST /api/v1/admin/announcements`

Protected: `admin`

### `GET /api/v1/admin/verifications`

Protected: `admin`

Returns verification queue items:

```json
{
  "id": "df9b1e9c-4ef8-47f1-8d77-c9ac75e520a8",
  "userId": "0bf4f4d5-48a9-4b7b-aaf4-77e805fc2902",
  "userEmail": "agent@example.com",
  "userRole": "agent",
  "status": "submitted",
  "submittedAt": "2026-04-06T09:05:00Z",
  "reviewedAt": null,
  "rejectionReason": null,
  "notes": "Agent KYC submitted",
  "createdAt": "2026-04-06T09:05:00Z",
  "updatedAt": "2026-04-06T09:05:00Z"
}
```

### `PATCH /api/v1/admin/verifications/{id}`

Protected: `admin`

Request:

```json
{
  "verification_status": "verified",
  "verification_notes": "Approved by admin"
}
```

Notes:

- compatibility mapping is applied:
  - `verified` -> verification record becomes `approved`
  - user record becomes legacy `verified`
  - KYC status email is sent

## Notifications

### `GET /api/v1/notifications`

Protected.

### `PATCH /api/v1/notifications/read-all`

Protected.

### `PATCH /api/v1/notifications/{id}/read`

Protected.

### `DELETE /api/v1/notifications/{id}`

Protected.

## Uploads

### `POST /api/v1/uploads/presign`

Request:

```json
{
  "category": "property-media",
  "filename": "front.jpg",
  "contentType": "image/jpeg"
}
```

## Legacy Routes Still Available

The backend still exposes older non-prefixed routes used by the earlier backend surface. These remain available for backward compatibility:

- `/health`
- `/auth/register`
- `/auth/login`
- `/auth/verify-email`
- `/dashboard`
- `/properties`
- `/posts`
- `/posts/{id}/respond`
- `/responses/{id}/thread`
- `/responses/{id}/live-video-sessions`
- `/responses/{id}/site-visits`
- `/reviews`
- `/reports`
- `/admin/reports/{id}/decision`

## Verification Status Notes

Two verification representations currently coexist:

- compatibility verification records:
  - `not_started`
  - `submitted`
  - `in_review`
  - `approved`
  - `rejected`
  - `expired`
- legacy user listing verification:
  - `not_required`
  - `pending`
  - `verified`
  - `rejected`

The `/api/v1/admin/verifications/{id}` endpoint maps between them automatically.
