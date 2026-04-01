use crate::{
    application::services::AuthService,
    domain::users::{AuthResponse, BootstrapAdminInput, LoginInput, RegisterUserInput},
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
}
