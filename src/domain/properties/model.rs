use crate::domain::users::UserRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[sqlx(type_name = "property_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PropertyStatus {
    Draft,
    Published,
}

#[derive(Debug, Deserialize)]
pub struct CreatePropertyInput {
    pub title: String,
    pub price: i64,
    pub location: String,
    pub exact_address: String,
    pub description: String,
    pub images: Vec<String>,
    pub contact_name: String,
    pub contact_phone: String,
    pub is_service_apartment: bool,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct PropertyQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub location: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct Property {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub title: String,
    pub price: i64,
    pub location: String,
    pub exact_address: String,
    pub description: String,
    pub images: Vec<String>,
    pub contact_name: String,
    pub contact_phone: String,
    pub is_service_apartment: bool,
    pub status: PropertyStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PropertyListItem {
    pub id: Uuid,
    pub title: String,
    pub price: i64,
    pub location: String,
    pub description: String,
    pub images: Vec<String>,
    pub is_service_apartment: bool,
    pub owner_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub owner_name: String,
    pub agent_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PropertyDetail {
    pub id: Uuid,
    pub title: String,
    pub price: i64,
    pub location: String,
    pub description: String,
    pub images: Vec<String>,
    pub is_service_apartment: bool,
    pub owner_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub owner_name: String,
    pub agent_name: Option<String>,
    pub exact_address: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PropertyDetail {
    pub fn sanitize_for_role(mut self, role: UserRole) -> Self {
        if matches!(role, UserRole::Buyer) {
            self.exact_address = None;
            self.contact_name = None;
            self.contact_phone = None;
        }
        self
    }
}
