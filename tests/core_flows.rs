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
            "role": "seeker",
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

#[tokio::test]
async fn login_creates_email_verification_token_and_verify_endpoint_marks_user_verified() {
    let _guard = test_mutex().lock().await;
    let app = setup_app().await;

    let user = register_user(
        &app,
        json!({
            "full_name": "Buyer Verify",
            "email": "verify@test.local",
            "password": "StrongPass123",
            "role": "seeker"
        }),
    )
    .await;
    assert_eq!(user["user"]["email_verified"], false);

    let login = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "email": "verify@test.local",
                    "password": "StrongPass123"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(login.0, StatusCode::OK);

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://verinest:verinest@127.0.0.1:55432/verinest_test".to_string());
    let pool = create_pool(&database_url, 1).await.expect("db pool");
    let token: String = sqlx::query_scalar(
        r#"
        SELECT evt.token
        FROM email_verification_tokens evt
        JOIN users u ON u.id = evt.user_id
        WHERE u.email = $1
        ORDER BY evt.created_at DESC
        LIMIT 1
        "#,
    )
    .bind("verify@test.local")
    .fetch_one(&pool)
    .await
    .expect("verification token");

    let verified = request_json(
        &app,
        Request::builder()
            .method("GET")
            .uri(format!("/auth/verify-email?token={token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(verified.0, StatusCode::OK);
    assert_eq!(verified.1["email_verified"], true);
}

#[tokio::test]
async fn api_v1_unassigned_onboarding_and_role_surfaces_work() {
    let _guard = test_mutex().lock().await;
    let app = setup_app().await;

    let register = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "full_name": "Seeker Compat",
                    "email": "compat-seeker@test.local",
                    "password": "StrongPass123",
                    "phone": "+2348011111111",
                    "bio": "Testing compatibility flow"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(register.0, StatusCode::CREATED);
    assert_eq!(register.1["user"]["role"], "unassigned");
    let token = token_of(&register.1);

    let send_code = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/send-email-code")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "email": "compat-seeker@test.local",
                    "purpose": "signup"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(send_code.0, StatusCode::OK);

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://verinest:verinest@127.0.0.1:55432/verinest_test".to_string());
    let pool = create_pool(&database_url, 1).await.expect("db pool");
    let code: String = sqlx::query_scalar(
        r#"
        SELECT code
        FROM email_verification_codes
        WHERE email = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind("compat-seeker@test.local")
    .fetch_one(&pool)
    .await
    .expect("email code");

    let verify_code = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/verify-email-code")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "email": "compat-seeker@test.local",
                    "code": code
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(verify_code.0, StatusCode::OK);
    assert_eq!(verify_code.1["email_verified"], true);

    let select_role = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/onboarding/role")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(json!({ "role": "seeker" }).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(select_role.0, StatusCode::OK);
    assert_eq!(select_role.1["user"]["role"], "seeker");

    let onboarding = request_json(
        &app,
        Request::builder()
            .method("PUT")
            .uri("/api/v1/onboarding/profile")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "phone": "+2348011111111",
                    "city": "Lagos",
                    "preferredCity": "Lagos",
                    "preferredAccommodationType": "Rent",
                    "preferredBudgetLabel": "500k-1m",
                    "moveInTimeline": "Within 1 month"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(onboarding.0, StatusCode::OK);
    assert_eq!(onboarding.1["profile"]["onboarding_completed"], true);

    let me = request_json(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(me.0, StatusCode::OK);
    assert_eq!(me.1["user"]["role"], "seeker");

    let need = request_json(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/seeker/needs")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "request_title": "Need 2 bed in Yaba",
                    "area": "Yaba",
                    "city": "Lagos",
                    "state": "Lagos",
                    "property_type": "rent",
                    "bedrooms": 2,
                    "min_budget": 700000,
                    "max_budget": 1500000,
                    "pricing_preference": "yearly",
                    "desired_features": ["parking", "security"],
                    "description": "Close to work"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(need.0, StatusCode::CREATED);

    let overview = request_json(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/v1/seeker/dashboard/overview")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(overview.0, StatusCode::OK);
    assert_eq!(overview.1["stats"]["needCount"], 1);

    let user_id = uuid::Uuid::parse_str(register.1["user"]["id"].as_str().expect("user id"))
        .expect("valid user id");
    sqlx::query(
        r#"
        INSERT INTO notifications (id, user_id, type, title, body, data_json)
        VALUES ($1, $2, 'info', 'Welcome', 'Compatibility flow', '{"actionUrl":"/seeker"}'::jsonb)
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("insert notification");

    let notifications = request_json(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/v1/notifications")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(notifications.0, StatusCode::OK);
    assert_eq!(notifications.1.as_array().unwrap().len(), 1);
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
        app_base_url: "http://127.0.0.1:3000".to_string(),
        mail_provider: "disabled".to_string(),
        mail_from_email: "noreply@test.local".to_string(),
        mail_from_name: "VeriNest Test".to_string(),
        resend_api_key: None,
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        smtp_use_starttls: true,
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
            notifications,
            announcements,
            disputes,
            calendar_events,
            maintenance_requests,
            payouts,
            transactions,
            rent_charges,
            leases,
            units,
            bookings,
            offers,
            lead_matches,
            saved_properties,
            verification_documents,
            verifications,
            landlord_profiles,
            agent_profiles,
            seeker_profiles,
            profiles,
            refresh_tokens,
            site_visit_certifications,
            site_visits,
            live_video_sessions,
            email_verification_codes,
            email_verification_tokens,
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
