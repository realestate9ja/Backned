use crate::{
    domain::{
        properties::{CreatePropertyInput, PropertyQuery},
        users::UserRole,
    },
    interfaces::http::{
        errors::AppError,
        middleware::{
            auth::{AuthUser, OptionalAuthUser},
            rbac::ensure_role,
        },
        state::AppState,
    },
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

pub async fn create_property(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreatePropertyInput>,
) -> Result<(StatusCode, Json<crate::domain::properties::PropertyDetail>), AppError> {
    ensure_role(&user, &[UserRole::Agent, UserRole::Landlord])?;
    let property = state.property_use_cases.create(&user, payload).await?;
    Ok((StatusCode::CREATED, Json(property)))
}

pub async fn list_properties(
    State(state): State<AppState>,
    optional_user: OptionalAuthUser,
    Query(query): Query<PropertyQuery>,
) -> Result<Json<Vec<crate::domain::properties::PropertyListItem>>, AppError> {
    let mut properties = state.property_use_cases.list(query).await?;
    
    // Sanitize sensitive data based on user role
    if let Some(user) = &optional_user.0 {
        properties = properties
            .into_iter()
            .map(|p| p.sanitize_for_role(user.role, Some(user.id)))
            .collect();
    } else {
        // Hide agent phone for unauthenticated users
        properties = properties
            .into_iter()
            .map(|p| p.sanitize_for_role(UserRole::Seeker, None))
            .collect();
    }
    
    Ok(Json(properties))
}

pub async fn get_property(
    State(state): State<AppState>,
    optional_user: OptionalAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::domain::properties::PropertyDetail>, AppError> {
    let property = state
        .property_use_cases
        .get_by_id(id, optional_user.0.as_ref())
        .await?;
    Ok(Json(property))
}
