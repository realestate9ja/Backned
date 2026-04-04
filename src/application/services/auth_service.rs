use crate::{
    domain::users::{
        AuthResponse, BootstrapAdminInput, LoginInput, RegisterUserInput, UserPublicView, UserRepository,
        UserRole, VerifyEmailInput,
    },
    infrastructure::{
        auth::{JwtService, PasswordService},
        cache::CacheService,
        email::MailService,
    },
    interfaces::http::errors::AppError,
    utils::validation,
};
use chrono::{Duration, Utc};

#[derive(Clone)]
pub struct AuthService {
    users: UserRepository,
    password_service: PasswordService,
    jwt_service: JwtService,
    cache: CacheService,
    mail_service: MailService,
    app_base_url: String,
}

impl AuthService {
    pub fn new(
        users: UserRepository,
        password_service: PasswordService,
        jwt_service: JwtService,
        cache: CacheService,
        mail_service: MailService,
        app_base_url: String,
    ) -> Self {
        Self {
            users,
            password_service,
            jwt_service,
            cache,
            mail_service,
            app_base_url,
        }
    }

    pub async fn register(&self, input: RegisterUserInput) -> Result<AuthResponse, AppError> {
        validation::validate_required(&input.full_name, "full_name")?;
        validation::validate_email(&input.email)?;
        validation::validate_password(&input.password)?;
        if input.role == UserRole::Admin {
            return Err(AppError::forbidden("admin users cannot be created through public registration"));
        }

        if self.users.find_by_email(&input.email).await?.is_some() {
            return Err(AppError::conflict("user with this email already exists"));
        }

        let password_hash = self.password_service.hash_password(&input.password)?;
        let user = self.users.create(&input, &password_hash).await?;
        if matches!(user.role, crate::domain::users::UserRole::Agent) {
            self.cache.invalidate_namespace("agents").await?;
        }
        self.build_auth_response(user).await
    }

    pub async fn bootstrap_admin(&self, input: BootstrapAdminInput) -> Result<AuthResponse, AppError> {
        validation::validate_required(&input.full_name, "full_name")?;
        validation::validate_email(&input.email)?;
        validation::validate_password(&input.password)?;

        if let Some(existing) = self.users.find_by_email(&input.email).await? {
            if existing.role == UserRole::Admin {
                return Err(AppError::conflict("admin with this email already exists"));
            }
            return Err(AppError::conflict("user with this email already exists"));
        }

        let password_hash = self.password_service.hash_password(&input.password)?;
        let user = self.users.create_admin(&input, &password_hash).await?;
        self.build_auth_response(user).await
    }

    pub async fn login(&self, input: LoginInput) -> Result<AuthResponse, AppError> {
        validation::validate_email(&input.email)?;
        validation::validate_required(&input.password, "password")?;

        let user = self
            .users
            .find_by_email(&input.email)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid credentials"))?;
        if user.is_banned {
            return Err(AppError::forbidden("account is banned"));
        }

        let valid = self
            .password_service
            .verify_password(&input.password, &user.password_hash)?;

        if !valid {
            return Err(AppError::unauthorized("invalid credentials"));
        }

        if !user.email_verified {
            let verification_token = self.users.create_email_verification_token(user.id).await?;
            let verification_link = format!(
                "{}/auth/verify-email?token={}",
                self.app_base_url.trim_end_matches('/'),
                verification_token
            );
            let email = self
                .mail_service
                .verification_email(user.email.clone(), &user.full_name, &verification_link);
            self.mail_service.send(email).await?;
        }

        self.build_auth_response(user).await
    }

    pub async fn verify_email(&self, input: VerifyEmailInput) -> Result<UserPublicView, AppError> {
        validation::validate_required(&input.token, "token")?;

        let user = self
            .users
            .find_by_email_verification_token(&input.token)
            .await?
            .ok_or_else(|| AppError::bad_request("invalid or expired verification token"))?;

        let updated = self
            .users
            .mark_email_verified(user.id)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))?;
        self.users.mark_email_verification_token_used(&input.token).await?;

        Ok(UserPublicView::from(updated))
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        validation::validate_required(refresh_token, "refresh_token")?;

        let user = self
            .users
            .consume_refresh_token(refresh_token)
            .await?
            .ok_or_else(|| AppError::unauthorized("invalid refresh token"))?;
        if user.is_banned {
            return Err(AppError::forbidden("account is banned"));
        }

        self.build_auth_response(user).await
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        validation::validate_required(refresh_token, "refresh_token")?;
        self.users.revoke_refresh_token(refresh_token).await?;
        Ok(())
    }

    async fn build_auth_response(&self, user: crate::domain::users::User) -> Result<AuthResponse, AppError> {
        let token = self.jwt_service.generate_token(&user)?;
        let refresh_token = self
            .users
            .create_refresh_token(user.id, Utc::now() + Duration::days(30))
            .await?;
        Ok(AuthResponse {
            token,
            refresh_token,
            user: UserPublicView::from(user),
        })
    }
}
