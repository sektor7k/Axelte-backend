use axum::routing::get;
use axum::{middleware::from_fn_with_state, routing::post, Router};
use sqlx::MySqlPool;
use crate::middleware::auth_middleware::auth_middleware;
use crate::handlers::body_handlers::{create_page, create_workspace, delete_page, get_page, get_workspace_id, get_workspace_members, get_workspace_pages, get_workspaces, rename_page, update_page};
use crate::handlers::service_handler::{poll_job, scrape_url};
pub fn body_routes(pool:MySqlPool) -> Router{
    Router::new()
    .route("/createworkspace", post(create_workspace))
    .route("/get-workspaces", get(get_workspaces))
    .route("/workspaces/{workspaceId}", get(get_workspace_id))
    .route("/workspaces/{workspaceId}/pages", get(get_workspace_pages))
    .route("/workspaces/{workspaceId}/members", get(get_workspace_members))
    .route("/create-page", post(create_page))
    .route("/get-page/{pageId}", get(get_page))
    .route("/rename-page",post(rename_page))
    .route("/delete-page",post(delete_page))
    .route("/update-page",post(update_page))
    .route("/scrape",post(scrape_url))
    .route("/jobs/{id}",get(poll_job))
    .layer(from_fn_with_state(pool.clone(), auth_middleware))
}