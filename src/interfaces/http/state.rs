use crate::{
    application::{
        services::{
            AuditService, AuthService, PostService, PropertyService, TrustService, UserService,
            WorkflowService,
        },
        use_cases::{AuthUseCases, PostUseCases, PropertyUseCases, TrustUseCases, UserUseCases, WorkflowUseCases},
    },
    config::Settings,
    domain::{
        audit::AuditLogRepository, notifications::NotificationRepository, posts::PostRepository,
        properties::PropertyRepository, responses::ResponseRepository, trust::TrustRepository,
        users::UserRepository, workflow::WorkflowRepository,
    },
    infrastructure::{auth::{JwtService, PasswordService}, cache::CacheService, livekit::LiveKitService},
    infrastructure::rate_limit::RateLimiter,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub auth_use_cases: AuthUseCases,
    pub user_use_cases: UserUseCases,
    pub property_use_cases: PropertyUseCases,
    pub post_use_cases: PostUseCases,
    pub workflow_use_cases: WorkflowUseCases,
    pub trust_use_cases: TrustUseCases,
    pub audit_service: AuditService,
    pub jwt_service: JwtService,
    pub user_repository: UserRepository,
    pub admin_bootstrap_token: String,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(pool: PgPool, settings: Settings) -> Self {
        let user_repository = UserRepository::new(pool.clone());
        let property_repository = PropertyRepository::new(pool.clone());
        let post_repository = PostRepository::new(pool.clone());
        let response_repository = ResponseRepository::new(pool.clone());
        let notification_repository = NotificationRepository::new(pool.clone());
        let workflow_repository = WorkflowRepository::new(pool.clone());
        let trust_repository = TrustRepository::new(pool.clone());
        let audit_repository = AuditLogRepository::new(pool);

        let password_service = PasswordService;
        let jwt_service = JwtService::new(&settings);
        let cache_service =
            CacheService::new(&settings.redis_url, settings.cache_ttl_seconds).expect("invalid redis config");
        let livekit_service = LiveKitService::new(
            settings.livekit_url.clone(),
            settings.livekit_api_key.clone(),
            settings.livekit_api_secret.clone(),
            settings.livekit_token_ttl_minutes,
        );
        let audit_service = AuditService::new(audit_repository);
        let rate_limiter = RateLimiter::new(
            settings.auth_rate_limit_max_requests,
            settings.auth_rate_limit_window_seconds,
            settings.trust_rate_limit_max_requests,
            settings.trust_rate_limit_window_seconds,
        );

        let auth_service = AuthService::new(
            user_repository.clone(),
            password_service,
            jwt_service.clone(),
            cache_service.clone(),
        );
        let user_service = UserService::new(
            user_repository.clone(),
            property_repository.clone(),
            post_repository.clone(),
            response_repository.clone(),
            notification_repository.clone(),
            workflow_repository.clone(),
            trust_repository.clone(),
            cache_service.clone(),
        );
        let property_service = PropertyService::new(
            property_repository.clone(),
            user_repository.clone(),
            workflow_repository.clone(),
            cache_service.clone(),
        );
        let post_service = PostService::new(
            post_repository,
            response_repository.clone(),
            user_repository.clone(),
            property_repository.clone(),
            notification_repository,
            cache_service.clone(),
        );
        let workflow_service = WorkflowService::new(
            workflow_repository.clone(),
            response_repository.clone(),
            property_repository.clone(),
            user_repository.clone(),
            livekit_service,
            cache_service.clone(),
        );
        let trust_service = TrustService::new(
            trust_repository,
            user_repository.clone(),
            response_repository,
            property_repository.clone(),
            workflow_repository.clone(),
            cache_service.clone(),
        );

        Self {
            auth_use_cases: AuthUseCases::new(auth_service),
            user_use_cases: UserUseCases::new(user_service),
            property_use_cases: PropertyUseCases::new(property_service),
            post_use_cases: PostUseCases::new(post_service),
            workflow_use_cases: WorkflowUseCases::new(workflow_service),
            trust_use_cases: TrustUseCases::new(trust_service),
            audit_service,
            jwt_service,
            user_repository,
            admin_bootstrap_token: settings.admin_bootstrap_token,
            rate_limiter,
        }
    }
}
