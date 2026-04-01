use crate::interfaces::http::{
    handlers::{auth, health, posts, properties, trust, users, workflow},
    middleware::{
        audit::{audit_middleware, request_context_middleware},
        rate_limit::{auth_rate_limit_middleware, trust_rate_limit_middleware},
    },
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
        .route(
            "/auth/register",
            post(auth::register).route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_rate_limit_middleware,
            )),
        )
        .route(
            "/auth/login",
            post(auth::login).route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_rate_limit_middleware,
            )),
        )
        .route(
            "/admin/bootstrap",
            post(auth::bootstrap_admin).route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_rate_limit_middleware,
            )),
        )
        .route("/dashboard", get(users::get_dashboard))
        .route("/users/{id}", get(users::get_user))
        .route("/users/{id}/reviews", get(trust::list_user_reviews))
        .route("/agents", get(users::list_agents))
        .route("/admin/agents/{id}/verification", patch(users::update_agent_verification))
        .route(
            "/agents/me/notification-settings",
            patch(users::update_agent_notification_settings),
        )
        .route("/agents/me/post-alerts", get(users::list_agent_post_alerts))
        .route("/properties", post(properties::create_property).get(properties::list_properties))
        .route("/properties/{id}", get(properties::get_property))
        .route("/properties/{id}/verify", post(workflow::verify_property))
        .route("/properties/{id}/publish", post(workflow::publish_property))
        .route("/properties/{id}/agent-request", post(workflow::create_property_agent_request))
        .route("/properties/{id}/assign-agent", post(workflow::assign_property_agent))
        .route("/posts", post(posts::create_post).get(posts::list_posts))
        .route("/posts/{id}/respond", post(posts::respond_to_post))
        .route("/responses/{id}/thread", get(workflow::get_thread))
        .route("/responses/{id}/thread/messages", post(workflow::add_thread_message))
        .route("/responses/{id}/live-video-sessions", post(workflow::create_live_video_session))
        .route(
            "/live-video-sessions/{id}",
            get(workflow::get_live_video_session_access).patch(workflow::update_live_video_session),
        )
        .route("/responses/{id}/site-visits", post(workflow::create_site_visit))
        .route("/site-visits/{id}", patch(workflow::update_site_visit))
        .route("/site-visits/{id}/certify", post(workflow::certify_site_visit))
        .route(
            "/reviews",
            post(trust::create_review).route_layer(middleware::from_fn_with_state(
                state.clone(),
                trust_rate_limit_middleware,
            )),
        )
        .route(
            "/reports",
            post(trust::create_report).route_layer(middleware::from_fn_with_state(
                state.clone(),
                trust_rate_limit_middleware,
            )),
        )
        .route("/admin/reports/{id}/decision", post(trust::moderate_report))
        .layer(middleware::from_fn_with_state(state.clone(), audit_middleware))
        .layer(middleware::from_fn(request_context_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
