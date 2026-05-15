//! HTTP REST API client for MiningOS app-node

pub mod auth;
pub mod client;
pub mod endpoints;

pub use auth::OAuthConfig;
pub use client::MiningOsHttpClient;
pub use endpoints::*;
