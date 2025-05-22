use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use std::env;

pub async fn init_db() -> Result<MySqlPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}
