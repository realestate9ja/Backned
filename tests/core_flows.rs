use std::sync::OnceLock;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use realestate::{
    build_app,
    config::Settings,
    db::{create_pool, run_migrations},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::util::ServiceExt;

static TEST_MUTEX: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn test_mutex() -> &'static tokio::sync::Mutex<()> {
    TEST_MUTEX.get_or_init(|| tokio::sync::Mutex::new(()))
}

#[tokio::test]
async fn agent_verification_review_gating_and_moderation_flow() {
    let _guard = test_mutex().lock().await;
    let app = setup_app().await;

    let admin = bootstrap_admin(&app).await;
    let admin_token = token_of(&admin);

    let buyer = register_user(
        &app,
        json!({
            "full_name": "Buyer One",
            "email": "buyer@test.local",
            "password": "StrongPass123",
            "role": "buyer",
            "bio": "Buyer"
        }),
    )
    .await;
    let buyer_token = token_of(&buyer);

    let agent = register_user(
        &app,
        json!({
            "full_name": "Agent One",
            "email": "agent@test.local",
            "password": "StrongPass123",
            "role": "agent",
            "phone": "+2348010000001",
            "bio": "Agent"
        }),
    )
    .await;
    let agent_id = user_id_of(&agent);
    let agent_token = token_of(&agent);

    let forbidden_property = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/properties")
            .header("authorization", format!("Bearer {agent_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "title": "Blocked Until Verified",
                    "price": 3000000,
                    "location": "Wuse 2, Abuja",
                    "exact_address": "1 Street, Abuja",
                    "description": "Should fail",
                    "images": ["https://cdn.example.com/block.jpg"],
                    "contact_name": "Agent One",
                    "contact_phone": "+2348010000001",
                    "is_service_apartment": true
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(forbidden_property.0, StatusCode::FORBIDDEN);

    let verify_agent = request_json(
        &app,
        Request::builder()
            .method("PATCH")
            .uri(format!("/admin/agents/{agent_id}/verification"))
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "verification_status": "verified",
                    "verification_notes": "ID manually reviewed"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(verify_agent.0, StatusCode::OK);

    let property = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/properties")
            .header("authorization", format!("Bearer {agent_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "title": "Verified Listing",
                    "price": 4500000,
                    "location": "Wuse 2, Abuja",
                    "exact_address": "14 Crescent, Abuja",
                    "description": "Serviced apartment",
                    "images": ["https://cdn.example.com/front.jpg", "https://cdn.example.com/tour.mp4"],
                    "contact_name": "Agent One",
                    "contact_phone": "+2348010000001",
                    "is_service_apartment": true
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(property.0, StatusCode::CREATED);
    let property_id = property.1["id"].as_str().unwrap().to_string();

    let post = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/posts")
            .header("authorization", format!("Bearer {buyer_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "request_title": "Need serviced apartment",
                    "area": "Wuse 2",
                    "city": "Abuja",
                    "state": "FCT",
                    "property_type": "service_apartment",
                    "bedrooms": 2,
                    "min_budget": 3000000,
                    "max_budget": 5000000,
                    "pricing_preference": "monthly",
                    "desired_features": ["wifi", "parking"],
                    "description": "Looking for a serviced apartment"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(post.0, StatusCode::CREATED);
    let post_id = post.1["id"].as_str().unwrap().to_string();

    let response = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/posts/{post_id}/respond"))
            .header("authorization", format!("Bearer {agent_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "message": "I have a match for you",
                    "property_ids": [property_id]
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.0, StatusCode::CREATED);
    let response_id = response.1["id"].as_str().unwrap().to_string();

    let blocked_review = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/reviews")
            .header("authorization", format!("Bearer {buyer_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "reviewee_id": agent_id,
                    "property_id": property_id,
                    "response_id": response_id,
                    "rating": 5,
                    "comment": "Too early"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(blocked_review.0, StatusCode::FORBIDDEN);

    let video = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/responses/{response_id}/live-video-sessions"))
            .header("authorization", format!("Bearer {buyer_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "scheduled_at": "2026-04-01T18:00:00Z",
                    "tracking_notes": "Requested walkthrough"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(video.0, StatusCode::CREATED);
    let video_id = video.1["id"].as_str().unwrap().to_string();

    let access = request_json(
        &app,
        Request::builder()
            .method("GET")
            .uri(format!("/live-video-sessions/{video_id}"))
            .header("authorization", format!("Bearer {buyer_token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(access.0, StatusCode::OK);
    assert_eq!(access.1["session"]["provider"], "livekit");
    assert_eq!(access.1["server_url"], "ws://127.0.0.1:7880");
    assert!(!access.1["token"].as_str().unwrap().is_empty());

    let complete_video = request_json(
        &app,
        Request::builder()
            .method("PATCH")
            .uri(format!("/live-video-sessions/{video_id}"))
            .header("authorization", format!("Bearer {agent_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "status": "completed",
                    "started_at": "2026-04-01T18:00:00Z",
                    "ended_at": "2026-04-01T18:15:00Z",
                    "tracking_notes": "Completed"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(complete_video.0, StatusCode::OK);

    let allowed_review = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/reviews")
            .header("authorization", format!("Bearer {buyer_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "reviewee_id": agent_id,
                    "property_id": property_id,
                    "response_id": response_id,
                    "rating": 5,
                    "comment": "Walkthrough was accurate"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(allowed_review.0, StatusCode::CREATED);

    let fraud_report = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/reports")
            .header("authorization", format!("Bearer {buyer_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "reported_user_id": agent_id,
                    "property_id": property_id,
                    "response_id": response_id,
                    "violation_type": "fraud",
                    "reason": "fake_listing",
                    "details": "Fraud review for moderation"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(fraud_report.0, StatusCode::CREATED);
    let report_id = fraud_report.1["id"].as_str().unwrap().to_string();

    let moderate = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/admin/reports/{report_id}/decision"))
            .header("authorization", format!("Bearer {admin_token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "status": "upheld",
                    "review_notes": "Confirmed"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(moderate.0, StatusCode::OK);
}

#[tokio::test]
async fn auth_and_trust_routes_are_rate_limited() {
    let _guard = test_mutex().lock().await;
    let app = setup_app().await;

    for index in 0..10 {
        let response = request_json(
            &app,
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("x-forwarded-for", "198.51.100.10")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": format!("nobody{index}@example.com"),
                        "password": "wrong-password"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await;
        assert_eq!(response.0, StatusCode::UNAUTHORIZED);
    }

    let limited = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("x-forwarded-for", "198.51.100.10")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "email": "overflow@example.com",
                    "password": "wrong-password"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(limited.0, StatusCode::TOO_MANY_REQUESTS);
}

async fn setup_app() -> axum::Router {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://verinest:verinest@127.0.0.1:55432/verinest_test".to_string());
    let redis_url =
        std::env::var("TEST_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let settings = Settings {
        database_url: database_url.clone(),
        redis_url,
        cache_ttl_seconds: 60,
        jwt_secret: "test-secret".to_string(),
        jwt_expiration_minutes: 60,
        port: 0,
        database_max_connections: 5,
        admin_bootstrap_token: "bootstrap-secret".to_string(),
        livekit_url: "ws://127.0.0.1:7880".to_string(),
        livekit_api_key: "test-livekit-key".to_string(),
        livekit_api_secret: "test-livekit-secret".to_string(),
        livekit_token_ttl_minutes: 60,
        auth_rate_limit_max_requests: 10,
        auth_rate_limit_window_seconds: 60,
        trust_rate_limit_max_requests: 20,
        trust_rate_limit_window_seconds: 60,
    };

    let pool = create_pool(&database_url, settings.database_max_connections)
        .await
        .expect("db pool");
    reset_database(&pool).await;
    build_app(pool, settings)
}

async fn reset_database(pool: &PgPool) {
    run_migrations(pool).await.expect("migrations");
    sqlx::query(
        r#"
        TRUNCATE TABLE
            site_visit_certifications,
            site_visits,
            live_video_sessions,
            thread_messages,
            request_threads,
            property_agent_requests,
            response_properties,
            reviews,
            reports,
            responses,
            agent_post_notifications,
            posts,
            properties,
            audit_logs,
            users
        RESTART IDENTITY CASCADE
        "#,
    )
    .execute(pool)
    .await
    .expect("truncate");
}

async fn bootstrap_admin(app: &axum::Router) -> Value {
    let response = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/admin/bootstrap")
            .header("x-admin-bootstrap-token", "bootstrap-secret")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "full_name": "Admin One",
                    "email": "admin@test.local",
                    "password": "StrongPass123"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(response.0, StatusCode::CREATED);
    response.1
}

async fn register_user(app: &axum::Router, payload: Value) -> Value {
    let response = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(response.0, StatusCode::CREATED);
    response.1
}

fn token_of(value: &Value) -> String {
    value["token"].as_str().unwrap().to_string()
}

fn user_id_of(value: &Value) -> String {
    value["user"]["id"].as_str().unwrap().to_string()
}

async fn request_json(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = response.into_body().collect().await.expect("body").to_bytes();
    let body = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice::<Value>(&bytes).expect("json body")
    };
    (status, body)
}
