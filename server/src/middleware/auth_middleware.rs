use axum::{
    extract::Extension,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    body::Body,
};
use jsonwebtoken::errors::ErrorKind;
use tower_cookies::{CookieManagerLayer, Cookies};
use sqlx::MySqlPool;
use uuid::Uuid;
use crate::models::user::User;
use crate::handlers::jwt::verify_token;
pub async fn auth_middleware(
    Extension(pool): Extension<MySqlPool>,
    mut req: Request<Body>,
    next: Next,
) -> impl IntoResponse {

    // 1. tower_cookies katmanının eklediği Cookies objesi
    let cookies = match req.extensions().get::<Cookies>() {
        Some(c) => c.clone(),
        None => return (StatusCode::UNAUTHORIZED, "Cookie manager yok").into_response(),
    };

    // 2. axtoken çerezini al
    let cookie = match cookies.get("axtoken") {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "Token bulunamadı").into_response(),
    };
    let token = cookie.value().to_string();

    // 3. JWT doğrula
    let data = match verify_token(&token) {
        Ok(d) => d,
        Err(e) => {
            let msg = match *e.kind() {
                ErrorKind::ExpiredSignature => "Token süresi dolmuş",
                _ => "Token geçersiz",
            };
            return (StatusCode::UNAUTHORIZED, msg).into_response()
        }
    };

    // 4. user_id'yi parse et
    let user_id = match Uuid::parse_str(&data.claims.sub) {
        Ok(u) => u.to_string(),
        Err(_) => return (StatusCode::UNAUTHORIZED, "ID parse hatası").into_response(),
    };

    // 5. DB'den kullanıcıyı çek
    let row = match sqlx::query!(
        "SELECT id, username, email, avatar, role FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Kullanıcı bulunamadı").into_response(),
    };

    // 6. User objesini request'e ekle
    req.extensions_mut().insert(User {
        id:       row.id,
        username: row.username,
        email:    row.email,
        avatar:   row.avatar,
        role:     row.role,
    });

    // 7. Handler zincirine devam et
    next.run(req).await
}
