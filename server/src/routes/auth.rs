use axum::{routing::post, Router};
use sqlx::MySqlPool;
use crate::handlers::auth_handlers::{signup,login,logout};
pub fn auth_routes(pool:MySqlPool) -> Router{
    Router::new()

    .route("/signup", post(signup))
    .route("/login", post(login))
    .route("/logout", post(logout))
    .with_state(pool)

}