use crate::domain::audit::{AuditLogRepository, CreateAuditLogInput};
use anyhow::Result;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditService {
    repository: AuditLogRepository,
}

#[derive(Clone)]
pub struct AuditActor {
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Clone)]
pub struct AuditEvent {
    pub request_id: Uuid,
    pub action: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub success: bool,
    pub metadata: Value,
}

impl AuditService {
    pub fn new(repository: AuditLogRepository) -> Self {
        Self { repository }
    }

    pub async fn record(&self, actor: AuditActor, event: AuditEvent) -> Result<()> {
        let input = CreateAuditLogInput {
            request_id: event.request_id,
            user_id: actor.user_id,
            email: actor.email,
            role: actor.role,
            action: event.action,
            method: event.method,
            path: event.path,
            status_code: i32::from(event.status_code),
            ip_address: event.ip_address,
            user_agent: event.user_agent,
            resource_type: event.resource_type,
            resource_id: event.resource_id,
            success: event.success,
            metadata: event.metadata,
        };

        self.repository.create(&input).await
    }
}
