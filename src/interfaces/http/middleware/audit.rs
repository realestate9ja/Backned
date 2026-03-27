use axum::{
    extract::{MatchedPath, State},
    http::{HeaderMap, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    application::services::{AuditActor, AuditEvent},
    domain::users::User,
    interfaces::http::{middleware::request_context::RequestContext, state::AppState},
};

pub async fn request_context_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let context = RequestContext::from_parts(&parts);
    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(context.clone());
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&context.request_id.to_string()) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}

pub async fn audit_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or(path.as_str())
        .to_string();
    let context = request
        .extensions()
        .get::<RequestContext>()
        .cloned()
        .unwrap_or_else(|| RequestContext {
            request_id: Uuid::new_v4(),
            ip_address: None,
            user_agent: None,
        });
    let actor = resolve_actor(&state, request.headers()).await;
    let response = next.run(request).await;

    if !matches!(matched_path.as_str(), "/auth/register" | "/auth/login") {
        let event = AuditEvent {
            request_id: context.request_id,
            action: action_name(&method, &matched_path),
            method,
            path,
            status_code: response.status().as_u16(),
            ip_address: context.ip_address,
            user_agent: context.user_agent,
            resource_type: resource_type(&matched_path),
            resource_id: None,
            success: response.status().is_success(),
            metadata: json!({ "matched_path": matched_path }),
        };

        if let Err(error) = state.audit_service.record(actor, event).await {
            tracing::error!("failed to persist audit log: {error:#}");
        }
    }

    response
}

async fn resolve_actor(state: &AppState, headers: &HeaderMap) -> AuditActor {
    match resolve_user_from_headers(state, headers).await {
        Some(user) => AuditActor {
            user_id: Some(user.id),
            email: Some(user.email.clone()),
            role: Some(role_name(&user)),
        },
        None => AuditActor {
            user_id: None,
            email: None,
            role: None,
        },
    }
}

async fn resolve_user_from_headers(state: &AppState, headers: &HeaderMap) -> Option<User> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)?;
    let claims = state.jwt_service.decode_token(token).ok()?;
    state.user_repository.find_by_id(claims.sub).await.ok().flatten()
}

fn role_name(user: &User) -> String {
    serde_json::to_string(&user.role)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

fn action_name(method: &str, matched_path: &str) -> String {
    match (method, matched_path) {
        ("GET", "/users/{id}") => "user.view".to_string(),
        ("GET", "/agents") => "agent.list".to_string(),
        ("POST", "/properties") => "property.create".to_string(),
        ("GET", "/properties") => "property.list".to_string(),
        ("GET", "/properties/{id}") => "property.view".to_string(),
        ("POST", "/posts") => "post.create".to_string(),
        ("GET", "/posts") => "post.list".to_string(),
        ("POST", "/posts/{id}/respond") => "post.respond".to_string(),
        _ => format!("{}.{}", matched_path.replace('/', ".").trim_matches('.'), method.to_lowercase()),
    }
}

fn resource_type(matched_path: &str) -> Option<String> {
    match matched_path {
        "/users/{id}" => Some("user".to_string()),
        "/agents" => Some("agent".to_string()),
        "/properties" | "/properties/{id}" => Some("property".to_string()),
        "/posts" | "/posts/{id}/respond" => Some("post".to_string()),
        _ => None,
    }
}
