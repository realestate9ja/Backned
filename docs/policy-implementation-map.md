# Policy To Implementation Map

## Implemented Database Changes

- `properties.status` now supports `suspended`
- `users` now stores:
  - `quality_strikes`
  - `fraud_strikes`
  - `listing_restricted_until`
  - `is_banned`
- `reports` now stores:
  - `violation_type`
  - `reviewed_by`
  - `reviewed_at`
  - `review_notes`
  - moderation status values `open | reviewing | upheld | dismissed`

Migration files:

- [migrations/0005_workflow_and_trust.sql](/home/dave/Code/realestate/migrations/0005_workflow_and_trust.sql)
- [migrations/0006_listing_policy_and_moderation.sql](/home/dave/Code/realestate/migrations/0006_listing_policy_and_moderation.sql)

## Implemented Runtime Rules

### Property creation

- Agent creates property:
  - property is immediately `published`
  - agent is auto-assigned as manager
- Landlord creates property:
  - property is `pending_verification`
  - `self_managed=true` unless an agent request is supplied

### Review enforcement

- When a new review has `rating <= 2` and targets a property:
  - backend counts low-rated reviews for that property
  - at 3 low-rated reviews, property becomes `suspended`

### Report moderation

- Internal moderation endpoint:
  - `POST /admin/reports/{id}/decision`
  - authenticated by admin JWT
- Moderation decisions:
  - `upheld`
  - `dismissed`
- Upheld decision triggers:
  - quality strike or fraud strike increment on reported user
  - optional property suspension
  - optional listing restriction or account ban

## Implemented API Changes

- `POST /internal/reports/{id}/decision`
- `GET /live-video-sessions/{id}`
- existing public review and report endpoints remain:
  - `POST /reviews`
  - `POST /reports`

## Files Carrying The Policy Logic

- [src/application/services/property_service.rs](/home/dave/Code/realestate/src/application/services/property_service.rs)
- [src/application/services/trust_service.rs](/home/dave/Code/realestate/src/application/services/trust_service.rs)
- [src/domain/properties/repository.rs](/home/dave/Code/realestate/src/domain/properties/repository.rs)
- [src/domain/trust/repository.rs](/home/dave/Code/realestate/src/domain/trust/repository.rs)
- [src/interfaces/http/handlers/trust.rs](/home/dave/Code/realestate/src/interfaces/http/handlers/trust.rs)
- [src/interfaces/http/middleware/internal.rs](/home/dave/Code/realestate/src/interfaces/http/middleware/internal.rs)
