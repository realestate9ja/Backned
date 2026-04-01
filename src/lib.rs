pub mod application;
pub mod config;
pub mod db;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod utils;

use axum::Router;
use config::Settings;
use interfaces::http::{routes::create_router, state::AppState};
use sqlx::PgPool;

pub fn build_app(pool: PgPool, settings: Settings) -> Router {
    let state = AppState::new(pool, settings);
    create_router(state)
}
