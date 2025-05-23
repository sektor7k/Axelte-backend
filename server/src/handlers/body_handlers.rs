use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::MySqlPool;
use uuid::Uuid;
use sqlx::types::*;

use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkspacePayload {
    pub name: String,
    pub description: String,
    pub emails: Vec<String>,
}

pub async fn create_workspace(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateWorkspacePayload>,
) -> impl IntoResponse {
    let ws_id = Uuid::new_v4().to_string();

    let result = sqlx::query!(
        "INSERT INTO workspaces (id,name, description, owner_id) VALUES (?, ?, ?, ?)",
        ws_id,
        payload.name,
        payload.description,
        user.id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    });

    // Then add the creator as an owner in workspace_members
    let member_result = sqlx::query!(
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES (?, ?, 'owner')",
        ws_id,
        user.id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Failed to add workspace member" })),
        )
    });

    //TODO: send email to all emails

    let body = Json(json!({
        "message": "workspace created successfully",
        "workspaceId": ws_id
    }));

    (StatusCode::OK, body)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
}

pub async fn get_workspaces(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let workspaces = sqlx::query_as!(
        Workspace,
        "
        SELECT w.id, w.name, w.description, w.owner_id
        FROM workspaces w
        INNER JOIN workspace_members wm ON w.id = wm.workspace_id
        WHERE wm.user_id = ?
        ",
        user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;

    let body = Json(json!({
        "workspaces": workspaces
    }));

    Ok((StatusCode::OK, body))
}


#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceId {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
}

pub async fn get_workspace_id(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
    Path(workspace_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query!(
        "SELECT id, name, description, owner_id FROM workspaces WHERE id = ? AND owner_id = ?",
        workspace_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;
    let workspace: Vec<WorkspaceId> = result
        .into_iter()
        .map(|row| WorkspaceId {
            id: row.id,
            name: row.name,
            description: row.description,
            owner_id: row.owner_id,
        })
        .collect();

    let body = Json(json!({
        "workspace": workspace
    }));

    Ok((StatusCode::OK, body))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub workspace_id: String,
}

pub async fn get_workspace_pages(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
    Path(workspace_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Önce workspace'in kullanıcının üyesi olduğunu kontrol et
    let workspace_exists = sqlx::query!(
        "SELECT workspace_id FROM workspace_members WHERE workspace_id = ? AND user_id = ?",
        workspace_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;

    if workspace_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Workspace not found or you don't have access" })),
        ));
    }

    let pages = sqlx::query_as!(
        Page,
        "SELECT id, title, workspace_id FROM pages WHERE workspace_id = ?",
        workspace_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;

    Ok((StatusCode::OK, Json(pages)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub role: String,
}

pub async fn get_workspace_members(
    Extension(pool): Extension<MySqlPool>,
    Path(workspace_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let members = sqlx::query_as!(
        Member,
        "SELECT wm.user_id, u.username, u.email, u.avatar, wm.role 
         FROM workspace_members wm 
         INNER JOIN users u ON wm.user_id = u.id 
         WHERE wm.workspace_id = ?",
        workspace_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;


    Ok((StatusCode::OK, Json(members)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePagePayload {
    pub title: String,
    pub workspace_id: String,
}

pub async fn create_page(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreatePagePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Önce kullanıcının workspace'teki rolünü kontrol et
    let user_role = sqlx::query!(
        "SELECT role FROM workspace_members WHERE workspace_id = ? AND user_id = ?",
        payload.workspace_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error"})),
        )
    })?;

    // Kullanıcı workspace'e üye değilse veya viewer rolündeyse hata döndür
    match user_role {
        Some(role) if role.role == "viewer" => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "Viewers cannot create pages"})),
            ));
        }
        None => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "You don't have access to this workspace"})),
            ));
        }
        _ => {} // owner veya editor ise devam et
    }

    let page_id = Uuid::new_v4().to_string();
    let result = sqlx::query!(
        "INSERT INTO pages (id, title, workspace_id, created_by) VALUES (?, ?, ?, ?)",
        page_id,
        payload.title,
        payload.workspace_id,
        user.id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error"})),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Page created successfully",
        "page_id": page_id
         })),
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageResponse {
    pub id: String,
    pub title: String,
    pub workspace_id: String,
    pub created_by: String,
    pub content: Option<Value>,
}

pub async fn get_page(
    Extension(pool): Extension<MySqlPool>,
    Path(page_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {


    let page = sqlx::query_as!(
        PageResponse,
        "SELECT id, title, workspace_id, created_by,content FROM pages WHERE id = ? ",
        page_id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;

    Ok((StatusCode::OK, Json(page)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenamePagePayload {
    pub id: String,
    pub title: String,
}

pub async fn rename_page(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
    Json(payload): Json<RenamePagePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Önce sayfanın workspace_id'sini al
    let page = sqlx::query!(
        "SELECT workspace_id FROM pages WHERE id = ?",
        payload.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error"})),
        )
    })?;

    let workspace_id = match page {
        Some(p) => p.workspace_id,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Page not found"})),
        )),
    };

    // Kullanıcının workspace'teki rolünü kontrol et
    let user_role = sqlx::query!(
        "SELECT role FROM workspace_members WHERE workspace_id = ? AND user_id = ?",
        workspace_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error"})),
        )
    })?;

    // Kullanıcı workspace'e üye değilse veya viewer rolündeyse hata döndür
    match user_role {
        Some(role) if role.role == "viewer" => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "Viewers cannot rename pages"})),
            ));
        }
        None => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "You don't have access to this workspace"})),
            ));
        }
        _ => {} // owner veya editor ise devam et
    }

    let result = sqlx::query!(
        "UPDATE pages SET title = ? WHERE id = ?",
        payload.title,
        payload.id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;
    Ok((StatusCode::OK, Json(json!({ "message": "Page renamed successfully" }))))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePagePayload {
    pub id: String,
}

pub async fn delete_page(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
    Json(payload): Json<DeletePagePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Önce sayfanın workspace_id'sini al
    let page = sqlx::query!(
        "SELECT workspace_id FROM pages WHERE id = ?",
        payload.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error"})),
        )
    })?;

    let workspace_id = match page {
        Some(p) => p.workspace_id,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "Page not found"})),
        )),
    };

    // Kullanıcının workspace'teki rolünü kontrol et
    let user_role = sqlx::query!(
        "SELECT role FROM workspace_members WHERE workspace_id = ? AND user_id = ?",
        workspace_id,
        user.id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error"})),
        )
    })?;

    // Kullanıcı workspace'e üye değilse veya viewer rolündeyse hata döndür
    match user_role {
        Some(role) if role.role == "viewer" => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "Viewers cannot delete pages"})),
            ));
        }
        None => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "message": "You don't have access to this workspace"})),
            ));
        }
        _ => {} // owner veya editor ise devam et
    }

    let result = sqlx::query!(
        "DELETE FROM pages WHERE id = ?",
        payload.id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;
    Ok((StatusCode::OK, Json(json!({ "message": "Page deleted successfully" }))))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePagePayload {
    pub id: String,
    pub content: Value,
}

pub async fn update_page(
    Extension(pool): Extension<MySqlPool>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdatePagePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    

    let result = sqlx::query!(
        "UPDATE pages SET content = ? WHERE id = ?",
        sqlx::types::Json(payload.content),
        payload.id
    )
    .execute(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Database error" })),
        )
    })?;
    Ok((StatusCode::OK, Json(json!({ "message": "Page updated successfully" }))))
}