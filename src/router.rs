use axum::{Router, routing};

use crate::handlers::{auth, callback};

/// Return a full Axum router with both routes used by OAuth.
pub fn oauth_router() -> Router {
    Router::new()
        .route("/auth", routing::get(auth))
        .route("/callback", routing::get(callback))
}
