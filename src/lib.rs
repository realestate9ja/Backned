pub mod application;
pub mod config;
pub mod db;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod utils;

use axum::Router;
use interfaces::http::{routes::create_router, state::AppState};

pub fn build_app_with_state(state: AppState) -> Router {
    create_router(state)
}
