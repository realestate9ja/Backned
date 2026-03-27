use crate::{
    domain::users::{AuthResponse, LoginInput, RegisterUserInput, UserPublicView, UserRepository},
    infrastructure::{auth::{JwtService, PasswordService}, cache::CacheService},
    interfaces::http::errors::AppError,
    utils::validation,
};

#[derive(Clone)]
pub struct AuthService {
    users: UserRepository,
    password_service: PasswordService,
    jwt_service: JwtService,
    cache: CacheService,
}

impl AuthService {
    pub fn new(
        users: UserRepository,
        password_service: PasswordService,
        jwt_service: JwtService,
        cache: CacheService,
    ) -> Self {
        Self {
            users,
            password_service,
            jwt_service,
            cache,
        }
    }

    pub async fn register(&self, input: RegisterUserInput) -> Result<AuthResponse, AppError> {
        validation::validate_required(&input.full_name, "full_name")?;
        validation::validate_email(&input.email)?;
        validation::validate_password(&input.password)?;

        if self.users.find_by_email(&input.email).await?.is_some() {
            return Err(AppError::conflict("user with this email already exists"));
        }

        let password_hash = self.password_service.hash_password(&input.password)?;
        let user = self.users.create(&input, &password_hash).await?;
        if matches!(user.role, crate::domain::users::UserRole::Agent) {
            self.cache.invalidate_namespace("agents").await?;
        }
        let token = self.jwt_service.generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: UserPublicView::from(user),
        })
    }

    pub async fn login(&self, input: LoginInput) -> Result<AuthResponse, AppError> {
        validation::validate_email(&input.email)?;
        validation::validate_required(&input.password, "password")?;

        let user = self
            .users
            .find_by_email(&input.email)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid credentials"))?;

        let valid = self
            .password_service
            .verify_password(&input.password, &user.password_hash)?;

        if !valid {
            return Err(AppError::unauthorized("invalid credentials"));
        }

        let token = self.jwt_service.generate_token(&user)?;
        Ok(AuthResponse {
            token,
            user: UserPublicView::from(user),
        })
    }
}
