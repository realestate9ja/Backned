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
    infrastructure::{
        auth::{JwtService, PasswordService},
        cache::CacheService,
        email::MailService,
        livekit::LiveKitService,
    },
    infrastructure::rate_limit::RateLimiter,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub auth_use_cases: AuthUseCases,
    pub user_use_cases: UserUseCases,
    pub property_use_cases: PropertyUseCases,
    pub post_use_cases: PostUseCases,
    pub workflow_use_cases: WorkflowUseCases,
    pub trust_use_cases: TrustUseCases,
    pub audit_service: AuditService,
    pub jwt_service: JwtService,
    pub user_repository: UserRepository,
    pub mail_service: MailService,
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
        let audit_repository = AuditLogRepository::new(pool.clone());

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
        let mail_service = build_mail_service(&settings).expect("invalid mail config");
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
            mail_service.clone(),
            settings.app_base_url.clone(),
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
            mail_service.clone(),
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
            pool,
            auth_use_cases: AuthUseCases::new(auth_service),
            user_use_cases: UserUseCases::new(user_service),
            property_use_cases: PropertyUseCases::new(property_service),
            post_use_cases: PostUseCases::new(post_service),
            workflow_use_cases: WorkflowUseCases::new(workflow_service),
            trust_use_cases: TrustUseCases::new(trust_service),
            audit_service,
            jwt_service,
            user_repository,
            mail_service,
            admin_bootstrap_token: settings.admin_bootstrap_token,
            rate_limiter,
        }
    }
}

fn build_mail_service(settings: &Settings) -> anyhow::Result<MailService> {
    match settings.mail_provider.trim().to_lowercase().as_str() {
        "disabled" => Ok(MailService::disabled(
            settings.mail_from_email.clone(),
            settings.mail_from_name.clone(),
        )),
        "resend" => Ok(MailService::resend(
            settings.mail_from_email.clone(),
            settings.mail_from_name.clone(),
            settings
                .resend_api_key
                .clone()
                .ok_or_else(|| anyhow::anyhow!("RESEND_API_KEY is required when MAIL_PROVIDER=resend"))?,
        )),
        "smtp" => MailService::smtp(
            settings.mail_from_email.clone(),
            settings.mail_from_name.clone(),
            settings
                .smtp_host
                .clone()
                .ok_or_else(|| anyhow::anyhow!("SMTP_HOST is required when MAIL_PROVIDER=smtp"))?,
            settings.smtp_port,
            settings
                .smtp_username
                .clone()
                .ok_or_else(|| anyhow::anyhow!("SMTP_USERNAME is required when MAIL_PROVIDER=smtp"))?,
            settings
                .smtp_password
                .clone()
                .ok_or_else(|| anyhow::anyhow!("SMTP_PASSWORD is required when MAIL_PROVIDER=smtp"))?,
            settings.smtp_use_starttls,
        ),
        provider => Err(anyhow::anyhow!("unsupported MAIL_PROVIDER: {provider}")),
    }
}
