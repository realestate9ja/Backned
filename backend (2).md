# VeriNest Rust Backend Specification

## Objective

Build a Rust backend that serves the existing frontend through real business flows, not mock UI data.

This document is flow-first:

- domain flows
- Rust module boundaries
- commands and queries
- durable entity shapes
- endpoint inventory

It intentionally avoids backend responses shaped like dashboard cards, labels, or demo widgets.

Base URL:

- `https://api.verinest.xyz/api/v1`

Primary business roles:

- `seeker`
- `agent`
- `landlord`
- `admin`
- `unassigned`

Frontend naming note:

- frontend route `/provider/*` maps to backend role `agent`



## Domain Model

Core entities:

- `User`
- `Profile`
- `EmailVerificationCode`
- `EmailVerificationToken`
- `RefreshToken`
- `Property`
- `PropertyMedia`
- `PropertyUnit`
- `SeekerNeed`
- `Offer`
- `SavedProperty`
- `Booking`
- `Verification`
- `VerificationDocument`
- `CollectionRecord`
- `Payout`
- `MaintenanceRequest`
- `CalendarEvent`
- `Announcement`
- `Notification`
- `Dispute`
- `Report`
- `AuditLog`

Role-owned aggregates:

- `SeekerProfile`
- `AgentProfile`
- `LandlordProfile`

## Database Tables

Identity:

- `users`
- `profiles`
- `refresh_tokens`
- `email_verification_codes`
- `email_verification_tokens`
- `password_reset_tokens`

User role rule:

- `users.role` should default to `unassigned`
- allowed values:
  - `unassigned`
  - `seeker`
  - `agent`
  - `landlord`
  - `admin`

Role profiles:

- `seeker_profiles`
- `agent_profiles`
- `agent_specializations`
- `landlord_profiles`
- `landlord_property_types`

Marketplace:

- `properties`
- `property_media`
- `property_units`
- `seeker_needs`
- `offers`
- `saved_properties`
- `bookings`
- `lead_matches`

Landlord operations:

- `leases`
- `rent_collections`
- `payouts`
- `maintenance_requests`
- `calendar_events`

Trust and admin:

- `verifications`
- `verification_documents`
- `notifications`
- `announcements`
- `transactions`
- `disputes`
- `reports`
- `audit_logs`

## Flow Map

### 1. Authentication Flow

Flow:

1. register account
2. send verification code and verification link
3. verify email
4. load current session via `auth/me`
5. if role is `unassigned`, continue to onboarding
6. sign in again later after role is assigned
7. refresh token
8. logout

Required endpoints:

- `POST /auth/register`
- `POST /auth/login`
- `POST /auth/send-email-code`
- `POST /auth/verify-email-code`
- `GET /auth/verify-email`
- `POST /auth/refresh`
- `POST /auth/logout`
- `GET /auth/me`

### 2. Onboarding Flow

Flow:

1. user enters onboarding with `role = unassigned`
2. choose role
3. submit shared profile
4. submit role-specific profile
5. for agent and landlord: start verification
6. upload verification documents
7. submit verification

Required endpoints:

- `PUT /onboarding/profile`
- `POST /verifications`
- `POST /uploads/presign`
- `POST /verifications/{id}/documents`
- `GET /verifications/me`

### 3. Public Discovery Flow

Flow:

1. list public properties
2. get property detail

Required endpoints:

- `GET /properties`
- `GET /properties/{id}`

### 4. Seeker Demand Flow

Flow:

1. create need post
2. list own need posts
3. receive offers
4. save property
5. unsave property
6. create booking/viewing
7. list bookings and viewings

Required endpoints:

- `POST /seeker/needs`
- `GET /seeker/needs`
- `GET /seeker/offers`
- `POST /seeker/saved-properties`
- `GET /seeker/saved-properties`
- `DELETE /seeker/saved-properties/{propertyId}`
- `POST /bookings`
- `GET /seeker/bookings`

### 5. Agent Supply Flow

Flow:

1. receive matched leads
2. inspect lead detail
3. create offer
4. manage listings
5. view payouts
6. view agent calendar

Required endpoints:

- `GET /agent/leads`
- `GET /agent/leads/{id}`
- `POST /offers`
- `GET /agent/properties`
- `POST /agent/properties`
- `PATCH /agent/properties/{id}`
- `GET /agent/payouts`
- `GET /agent/calendar`
- `GET /agent/bookings`

### 6. Landlord Operations Flow

Flow:

1. manage properties
2. manage units
3. monitor collections
4. view payouts
5. create and manage maintenance requests
6. view calendar events

Required endpoints:

- `GET /landlord/properties`
- `POST /landlord/properties`
- `GET /landlord/units`
- `POST /landlord/units`
- `GET /landlord/collections`
- `GET /landlord/payouts`
- `GET /landlord/maintenance`
- `POST /landlord/maintenance`
- `GET /landlord/calendar`

### 7. Admin Governance Flow

Flow:

1. view platform metrics
2. inspect users
3. inspect properties
4. inspect transactions
5. inspect disputes
6. inspect reports
7. review verifications
8. approve or reject verifications
9. publish announcements

Required endpoints:

- `GET /admin/metrics/overview`
- `GET /admin/users`
- `GET /admin/properties`
- `GET /admin/transactions`
- `GET /admin/disputes`
- `GET /admin/reports`
- `GET /admin/verifications`
- `PATCH /admin/verifications/{id}`
- `GET /admin/announcements`
- `POST /admin/announcements`

### 8. Notification Flow

Flow:

1. fetch notifications
2. mark notification read
3. mark all read
4. delete notification

Required endpoints:

- `GET /notifications`
- `POST /notifications/read-all`
- `PATCH /notifications/{id}/read`
- `DELETE /notifications/{id}`

## Commands and Queries

Use command/query separation in modules.

Examples:

- command:
  - `RegisterUser`
  - `SendEmailVerificationCode`
  - `VerifyEmailCode`
  - `CompleteOnboarding`
  - `CreateNeed`
  - `CreateOffer`
  - `CreateBooking`
  - `CreateVerification`
  - `UploadVerificationDocument`
- query:
  - `GetCurrentSession`
  - `ListPublicProperties`
  - `GetPropertyDetail`
  - `ListSeekerOffers`
  - `ListSeekerBookings`
  - `ListAgentLeads`
  - `GetAgentLeadDetail`
  - `ListLandlordUnits`
  - `ListAdminVerifications`

## Endpoint Contract

The backend should return domain data, not UI strings like:

- `label`
- `accent`
- `cardTitle`
- `note`

The frontend should derive those.

### Auth

#### `POST /auth/register`

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

Response:

```json
{
  "token": "jwt-token",
  "refresh_token": "refresh-token",
  "user": {
    "id": "uuid",
    "full_name": "Seeker One",
    "email": "seeker@example.com",
    "email_verified": false,
    "role": "unassigned",
    "bio": null,
    "average_rating": null,
    "review_count": 0,
    "verification_status": "not_required",
    "created_at": "2026-04-11T10:00:00Z"
  }
}
```

Rule:

- signup does not assign `seeker`, `agent`, or `landlord`
- every new account starts as `role = "unassigned"`
- role is committed during onboarding

#### `POST /auth/send-email-code`

Request:

```json
{
  "email": "seeker@example.com",
  "purpose": "verify_email"
}
```

Response:

```json
{
  "ok": true,
  "expires_in_seconds": 600,
  "code_length": 5
}
```

#### `POST /auth/verify-email-code`

Request:

```json
{
  "email": "seeker@example.com",
  "code": "12345"
}
```

Response:

```json
{
  "user": {
    "id": "uuid",
    "full_name": "Seeker One",
    "email": "seeker@example.com",
    "email_verified": true,
    "role": "unassigned",
    "verification_status": "not_required"
  }
}
```

#### `GET /auth/me`

Response:

```json
{
  "user": {
    "id": "uuid",
    "full_name": "User",
    "email": "user@example.com",
    "email_verified": true,
    "role": "unassigned",
    "bio": null,
    "average_rating": null,
    "review_count": 0,
    "verification_status": "pending",
    "created_at": "2026-04-11T10:00:00Z"
  },
  "profile": {
    "id": "uuid",
    "userId": "uuid",
    "fullName": "User",
    "phone": "+2348012345678",
    "city": "Lagos",
    "avatarUrl": null,
    "bio": null,
    "onboardingCompleted": true
  },
  "roleProfile": {},
  "verification": {
    "id": "uuid",
    "status": "submitted",
    "submittedAt": "2026-04-11T10:00:00Z",
    "reviewedAt": null,
    "rejectionReason": null,
    "notes": null
  }
}
```

Rules:

- `user.role` may be `unassigned`
- `profile.onboardingCompleted` may be `false`
- frontend route decision should be:
  - not verified -> confirm email
  - verified + unassigned -> onboarding
  - verified + assigned + onboarding incomplete -> onboarding
  - verified + assigned + onboarding complete -> role dashboard

### Onboarding

Role selection happens here, not at signup.

Required onboarding shape:

1. choose role
2. submit shared profile data
3. submit role-specific data
4. mark onboarding complete
5. for `agent` and `landlord`, optionally continue into verification flow

#### `POST /onboarding/role`

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

Response:

```json
{
  "user": {
    "id": "uuid",
    "full_name": "User",
    "email": "user@example.com",
    "email_verified": true,
    "role": "seeker",
    "verification_status": "not_required"
  },
  "profile": {
    "id": "uuid",
    "userId": "uuid",
    "fullName": "User",
    "phone": null,
    "city": null,
    "avatarUrl": null,
    "bio": null,
    "onboardingCompleted": false
  }
}
```

Rules:

- this endpoint can only be called when current role is `unassigned`
- once business data exists, role change should require admin tooling, not normal onboarding

#### `PUT /onboarding/profile`

Supports seeker, agent, landlord payloads.

Response:

- `user`
- `profile`
- `roleProfile`
- `verification`

### Public Properties

#### `GET /properties`

Response item minimum:

```json
{
  "id": "uuid",
  "title": "Modern 2 Bed, Ikoyi",
  "location": "Ikoyi, Lagos",
  "city": "Lagos",
  "state": "Lagos",
  "price": 3800000,
  "rentAmount": 3800000,
  "rentCurrency": "NGN",
  "rentPeriod": "year",
  "bedrooms": 2,
  "bathrooms": 2,
  "sqft": 1200,
  "propertyType": "Apartment",
  "images": ["https://cdn.example.com/property.jpg"],
  "averageRating": 4.8,
  "views": 234
}
```

### Seeker

#### `GET /seeker/dashboard/overview`

Response sections:

- `stats`
- `matchTrends`
- `savedProperties`
- `recentOffers`

#### `GET /seeker/needs`

Response item minimum:

- `id`
- `request_title`
- `area`
- `city`
- `state`
- `property_type`
- `bedrooms`
- `min_budget`
- `max_budget`
- `pricing_preference`
- `status`
- `offerCount`
- `createdAt`

#### `GET /seeker/offers`

Response item minimum:

- `id`
- `needPostId`
- `propertyId`
- `providerUserId`
- `providerRole`
- `propertyTitle`
- `offerPriceAmount`
- `offerPriceCurrency`
- `offerPricePeriod`
- `matchScore`
- `status`
- `message`
- `createdAt`

#### `GET /seeker/saved-properties`

Response item minimum:

- `propertyId`
- `title`
- `location`
- `priceAmount`
- `priceCurrency`
- `pricePeriod`
- `imageUrl`
- `savedAt`

#### `GET /seeker/bookings`

Response item minimum:

- `id`
- `bookingType`
- `status`
- `scheduledFor`
- `propertyId`
- `propertyTitle`
- `propertyLocation`
- `hostUserId`
- `hostName`
- `hostPhone`
- `amount`
- `paymentStatus`
- `notes`
- `createdAt`

### Agent

#### `GET /agent/dashboard/overview`

Response sections:

- `stats`
- `earningsSeries`
- `topListings`
- `recentLeads`

#### `GET /agent/leads`

Response item minimum:

- `id`
- `needPostId`
- `matchedPropertyId`
- `matchScore`
- `status`
- `slaExpiresAt`
- `createdAt`
- `updatedAt`
- `requestTitle`
- `location`
- `propertyType`
- `urgency`

#### `GET /agent/leads/{id}`

Must include:

- lead
- seeker need
- matched properties
- existing offer if any

#### `GET /agent/properties`

Response item minimum:

- `id`
- `title`
- `location`
- `price`
- `rentCurrency`
- `rentPeriod`
- `listingStatus`
- `views`
- `inquiries`
- `averageRating`
- `images`
- `createdAt`

#### `GET /agent/payouts`

Response item minimum:

- `id`
- `amount`
- `currency`
- `status`
- `reference`
- `releasedAt`
- `createdAt`

#### `GET /agent/calendar`

Response item minimum:

- `id`
- `eventType`
- `title`
- `startsAt`
- `endsAt`
- `status`
- `metadata`

### Landlord

#### `GET /landlord/dashboard/overview`

Response sections:

- `stats`
- `occupancySeries`
- `collectionSeries`
- `leaseExpiries`
- `maintenanceQueue`

#### `GET /landlord/units`

Response item minimum:

- `id`
- `propertyId`
- `unitCode`
- `name`
- `unitType`
- `bedroomsLabel`
- `rentAmount`
- `rentCurrency`
- `rentPeriod`
- `occupancyStatus`
- `listingStatus`
- `tenantUserId`
- `leaseId`
- `createdAt`
- `updatedAt`

#### `GET /landlord/collections`

Response item minimum:

- `id`
- `propertyId`
- `unitId`
- `amountExpected`
- `amountCollected`
- `currency`
- `dueDate`
- `status`

#### `GET /landlord/maintenance`

Response item minimum:

- `id`
- `propertyId`
- `unitId`
- `title`
- `description`
- `priority`
- `status`
- `reportedByUserId`
- `createdAt`

#### `GET /landlord/calendar`

Response item minimum:

- `id`
- `propertyId`
- `unitId`
- `eventType`
- `title`
- `startsAt`
- `endsAt`
- `status`
- `metadata`

### Verifications and Uploads

#### `POST /uploads/presign`

Response:

- `uploadUrl`
- `fileUrl`
- `fileKey`

#### `GET /verifications/me`

Response:

- `verification`
- `documents`

### Notifications

#### `GET /notifications`

Response item minimum:

- `id`
- `kind`
- `title`
- `body`
- `actionUrl`
- `readAt`
- `createdAt`

### Admin

#### `GET /admin/metrics/overview`

Response sections:

- `totalProperties`
- `activeUsers`
- `monthlyRevenue`
- `openDisputes`
- `revenueSeries`
- `propertyActivitySeries`
- `recentActivity`

#### `GET /admin/verifications`

Response item minimum:

- `id`
- `userId`
- `userEmail`
- `userRole`
- `status`
- `submittedAt`
- `reviewedAt`
- `rejectionReason`
- `notes`
- `createdAt`
- `updatedAt`

## Dedicated Overview Endpoints

Dedicated overview endpoints are allowed because they are aggregation queries, not domain writes.

Use:

- `GET /seeker/dashboard/overview`
- `GET /agent/dashboard/overview`
- `GET /landlord/dashboard/overview`
- `GET /admin/metrics/overview`

These should aggregate domain entities into dashboard-ready datasets, but still return domain-oriented fields.

## Build Order

1. bootstrap app, config, migrations
2. auth and verification code flow
3. `auth/me`
4. profiles and neutral-signup onboarding role assignment
5. public properties
6. seeker needs, offers, saved properties, bookings
7. agent leads, offers, listings, payouts, calendar
8. landlord properties, units, collections, maintenance, calendar
9. uploads and verifications
10. notifications
11. admin metrics and operational endpoints

## Definition of Done

This backend is aligned with the frontend when:

- signup uses email/password plus verification code
- signup creates `unassigned` users only
- onboarding is where role is chosen and committed
- login returns enough state for role-based routing
- onboarding writes role-specific data
- public properties page uses the real API
- seeker bookings and viewings are backed by one real bookings source
- provider inbox and offer flow are real
- landlord units, collections, maintenance, and calendar are real
- admin verification queue and announcements are real
- notifications are persisted
