use crate::interfaces::http::{
    handlers::{api_v1, auth, health, posts, properties, trust, users, workflow},
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
        .route("/auth/verify-email", get(auth::verify_email))
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
        .nest("/api/v1", create_api_v1_router(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), audit_middleware))
        .layer(middleware::from_fn(request_context_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn create_api_v1_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(api_v1::register).route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        )))
        .route("/auth/login", post(auth::login).route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        )))
        .route("/auth/verify-email", get(auth::verify_email))
        .route("/auth/send-email-code", post(api_v1::send_email_code).route_layer(
            middleware::from_fn_with_state(state.clone(), auth_rate_limit_middleware),
        ))
        .route("/auth/verify-email-code", post(api_v1::verify_email_code).route_layer(
            middleware::from_fn_with_state(state.clone(), auth_rate_limit_middleware),
        ))
        .route("/auth/me", get(api_v1::me))
        .route("/auth/refresh", post(api_v1::refresh).route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        )))
        .route("/auth/logout", post(api_v1::logout))
        .route("/onboarding/role", post(api_v1::select_onboarding_role))
        .route("/onboarding/profile", axum::routing::put(api_v1::upsert_onboarding_profile))
        .route("/verifications", post(api_v1::create_verification))
        .route("/verifications/me", get(api_v1::get_my_verification))
        .route("/verifications/{id}/documents", post(api_v1::create_verification_document))
        .route("/uploads/presign", post(api_v1::uploads_presign))
        .route("/properties", get(api_v1::list_public_properties))
        .route("/properties/{id}", get(properties::get_property))
        .route("/seeker/dashboard/overview", get(api_v1::seeker_dashboard_overview))
        .route("/agent/dashboard/overview", get(api_v1::agent_dashboard_overview))
        .route("/landlord/dashboard/overview", get(api_v1::landlord_dashboard_overview))
        .route("/agent/properties", post(properties::create_property).get(api_v1::list_agent_properties))
        .route("/agent/properties/{id}", patch(api_v1::update_agent_property))
        .route("/seeker/needs", post(api_v1::create_seeker_need).get(api_v1::list_seeker_needs))
        .route("/agent/leads", get(api_v1::list_agent_leads))
        .route("/agent/leads/{id}", get(api_v1::get_agent_lead_detail))
        .route("/agent/payouts", get(api_v1::list_agent_payouts))
        .route("/agent/calendar", get(api_v1::list_agent_calendar))
        .route("/offers", post(api_v1::create_offer))
        .route("/seeker/offers", get(api_v1::list_seeker_offers))
        .route("/seeker/saved-properties", post(api_v1::create_saved_property).get(api_v1::list_saved_properties))
        .route("/seeker/saved-properties/{propertyId}", axum::routing::delete(api_v1::delete_saved_property))
        .route("/bookings", post(api_v1::create_booking))
        .route("/seeker/bookings", get(api_v1::list_seeker_bookings))
        .route("/agent/bookings", get(api_v1::list_agent_bookings))
        .route("/landlord/properties", post(api_v1::create_landlord_property).get(api_v1::list_landlord_properties))
        .route("/landlord/units", post(api_v1::create_landlord_unit).get(api_v1::list_landlord_units))
        .route("/landlord/collections", get(api_v1::list_landlord_collections))
        .route("/landlord/payouts", get(api_v1::list_landlord_payouts))
        .route("/landlord/maintenance", post(api_v1::create_landlord_maintenance).get(api_v1::list_landlord_maintenance))
        .route("/landlord/calendar", get(api_v1::list_landlord_calendar))
        .route("/admin/metrics/overview", get(api_v1::admin_metrics_overview))
        .route("/admin/users", get(api_v1::list_admin_users))
        .route("/admin/properties", get(api_v1::list_admin_properties))
        .route("/admin/transactions", get(api_v1::list_admin_transactions))
        .route("/admin/disputes", get(api_v1::list_admin_disputes))
        .route("/admin/reports", get(api_v1::list_admin_reports))
        .route("/admin/announcements", get(api_v1::list_admin_announcements).post(api_v1::create_admin_announcement))
        .route("/admin/verifications", get(api_v1::admin_list_verifications))
        .route("/admin/verifications/{id}", patch(api_v1::admin_update_verification))
        .route("/notifications", get(api_v1::list_notifications))
        .route("/notifications/read-all", patch(api_v1::notifications_read_all))
        .route("/notifications/{id}/read", patch(api_v1::notification_mark_read))
        .route("/notifications/{id}", axum::routing::delete(api_v1::notification_delete))
        .with_state(state)
}
