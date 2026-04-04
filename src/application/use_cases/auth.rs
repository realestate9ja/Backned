use crate::{
    application::services::AuthService,
    domain::users::{AuthResponse, BootstrapAdminInput, LoginInput, RegisterUserInput, UserPublicView, VerifyEmailInput},
    interfaces::http::errors::AppError,
};

#[derive(Clone)]
pub struct AuthUseCases {
    service: AuthService,
}

impl AuthUseCases {
    pub fn new(service: AuthService) -> Self {
        Self { service }
    }

    pub async fn register(&self, input: RegisterUserInput) -> Result<AuthResponse, AppError> {
        self.service.register(input).await
    }

    pub async fn bootstrap_admin(&self, input: BootstrapAdminInput) -> Result<AuthResponse, AppError> {
        self.service.bootstrap_admin(input).await
    }

    pub async fn login(&self, input: LoginInput) -> Result<AuthResponse, AppError> {
        self.service.login(input).await
    }

    pub async fn verify_email(&self, input: VerifyEmailInput) -> Result<UserPublicView, AppError> {
        self.service.verify_email(input).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        self.service.refresh(refresh_token).await
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        self.service.logout(refresh_token).await
    }
}
