use crate::{
    application::{
        services::{AuditService, AuthService, PostService, PropertyService, UserService},
        use_cases::{AuthUseCases, PostUseCases, PropertyUseCases, UserUseCases},
    },
    config::Settings,
    domain::{
        audit::AuditLogRepository, posts::PostRepository, properties::PropertyRepository,
        responses::ResponseRepository, users::UserRepository,
    },
    infrastructure::{auth::{JwtService, PasswordService}, cache::CacheService},
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub auth_use_cases: AuthUseCases,
    pub user_use_cases: UserUseCases,
    pub property_use_cases: PropertyUseCases,
    pub post_use_cases: PostUseCases,
    pub audit_service: AuditService,
    pub jwt_service: JwtService,
    pub user_repository: UserRepository,
}

impl AppState {
    pub fn new(pool: PgPool, settings: Settings) -> Self {
        let user_repository = UserRepository::new(pool.clone());
        let property_repository = PropertyRepository::new(pool.clone());
        let post_repository = PostRepository::new(pool.clone());
        let response_repository = ResponseRepository::new(pool.clone());
        let audit_repository = AuditLogRepository::new(pool);

        let password_service = PasswordService;
        let jwt_service = JwtService::new(&settings);
        let cache_service =
            CacheService::new(&settings.redis_url, settings.cache_ttl_seconds).expect("invalid redis config");
        let audit_service = AuditService::new(audit_repository);

        let auth_service = AuthService::new(
            user_repository.clone(),
            password_service,
            jwt_service.clone(),
            cache_service.clone(),
        );
        let user_service = UserService::new(user_repository.clone(), cache_service.clone());
        let property_service =
            PropertyService::new(property_repository, user_repository.clone(), cache_service.clone());
        let post_service = PostService::new(post_repository, response_repository, cache_service.clone());

        Self {
            auth_use_cases: AuthUseCases::new(auth_service),
            user_use_cases: UserUseCases::new(user_service),
            property_use_cases: PropertyUseCases::new(property_service),
            post_use_cases: PostUseCases::new(post_service),
            audit_service,
            jwt_service,
            user_repository,
        }
    }
}
