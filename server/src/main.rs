mod routes;
mod db;
mod handlers;
mod models;
use std::env;
mod middleware;

use axum::{
    http::{header, HeaderValue, Method}, middleware::from_fn_with_state, routing::get, Extension, Router
};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{ CorsLayer};

use routes::{auth::auth_routes, body::{body_routes}};
use handlers::auth_handlers::me;
use middleware::auth_middleware::auth_middleware;


#[tokio::main]
async fn main(){

    dotenvy::dotenv().ok();
    let client_url = env::var("CLIENT_URL").expect("CLIENT_URL must be set");
    
    let pool = db::init_db().await.unwrap();

    let cors = CorsLayer::new()
    .allow_origin(client_url.parse::<HeaderValue>().unwrap()) 
    .allow_methods([Method::POST, Method::GET, Method::OPTIONS])
    .allow_headers([header::CONTENT_TYPE, header::COOKIE])
    .allow_credentials(true); 

    let protected = Router::new()
        .route("/me", get(me))
        // auth_middleware ile koru
        .layer(from_fn_with_state(pool.clone(), auth_middleware));

    
    let app = Router::new()
    .nest("/auth", auth_routes(pool.clone()))
    .nest("/api", protected,)
    .nest("/api", body_routes(pool.clone()))
     .layer(CookieManagerLayer::new())
    .layer(Extension(pool.clone()))
    .layer(cors);






    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


