use crate::{domain::users::{User, UserRole}, interfaces::http::errors::AppError};

pub fn ensure_role(user: &User, allowed: &[UserRole]) -> Result<(), AppError> {
    if allowed.iter().any(|role| *role == user.role) {
        Ok(())
    } else {
        Err(AppError::forbidden("insufficient permissions"))
    }
}

