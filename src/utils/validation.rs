use crate::interfaces::http::errors::AppError;

pub fn validate_required(value: &str, field: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::bad_request(format!("{field} is required")));
    }
    Ok(())
}

pub fn validate_email(email: &str) -> Result<(), AppError> {
    validate_required(email, "email")?;
    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(AppError::bad_request("email is invalid"));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::bad_request(
            "password must be at least 8 characters long",
        ));
    }
    Ok(())
}

pub fn validate_money(value: i64, field: &str) -> Result<(), AppError> {
    if value <= 0 {
        return Err(AppError::bad_request(format!("{field} must be greater than 0")));
    }
    Ok(())
}

pub fn validate_non_empty_vec(values: &[String], field: &str) -> Result<(), AppError> {
    if values.is_empty() || values.iter().any(|item| item.trim().is_empty()) {
        return Err(AppError::bad_request(format!("{field} must not be empty")));
    }
    Ok(())
}

