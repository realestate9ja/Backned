use uuid::Uuid;

pub struct CsrfService;

impl CsrfService {
    /// Generate a random CSRF token
    pub fn generate_token() -> String {
        Uuid::new_v4().to_string()
    }

    /// Validate tokens match using constant-time comparison
    pub fn validate(received: &str, expected: &str) -> bool {
        use subtle::ConstantTimeEq;
        received.as_bytes().ct_eq(expected.as_bytes()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_produces_uuid() {
        let token = CsrfService::generate_token();
        assert_eq!(token.len(), 36); // UUID v4 length with hyphens
        Uuid::parse_str(&token).expect("should be valid UUID");
    }

    #[test]
    fn test_validate_matching_tokens() {
        let token = "abc123";
        assert!(CsrfService::validate(token, token));
    }

    #[test]
    fn test_validate_different_tokens() {
        assert!(!CsrfService::validate("abc123", "xyz789"));
    }
}
