use crate::{
    domain::{properties::{CreatePropertyInput, PropertyQuery}, users::UserRole},
    interfaces::http::{
        errors::AppError,
        middleware::{auth::{AuthUser, OptionalAuthUser}, rbac::ensure_role},
        state::AppState,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
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
    Query(query): Query<PropertyQuery>,
) -> Result<Json<Vec<crate::domain::properties::PropertyListItem>>, AppError> {
    let properties = state.property_use_cases.list(query).await?;
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

