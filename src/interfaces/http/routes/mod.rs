use crate::interfaces::http::{
    handlers::{auth, health, posts, properties, users},
    middleware::audit::{audit_middleware, request_context_middleware},
    state::AppState,
};
use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/dashboard", get(users::get_dashboard))
        .route("/users/{id}", get(users::get_user))
        .route("/agents", get(users::list_agents))
        .route(
            "/agents/me/notification-settings",
            patch(users::update_agent_notification_settings),
        )
        .route("/agents/me/post-alerts", get(users::list_agent_post_alerts))
        .route("/properties", post(properties::create_property).get(properties::list_properties))
        .route("/properties/{id}", get(properties::get_property))
        .route("/posts", post(posts::create_post).get(posts::list_posts))
        .route("/posts/{id}/respond", post(posts::respond_to_post))
        .layer(middleware::from_fn_with_state(state.clone(), audit_middleware))
        .layer(middleware::from_fn(request_context_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
