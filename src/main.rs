use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, postgres::PgPoolOptions};
use std::env;

#[derive(Deserialize)]
struct UserPayload {
    name: String,
    email: String,
}

#[derive(Serialize, FromRow)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let db_url = env::var("DATABASE_URL").expect("Database url must be set");
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("failed to connect to database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("migrations failed");
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user).get(list_users))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on port 8080");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Welcome to the User management API"
}

async fn list_users(State(pool): State<PgPool>) -> Result<Json<Vec<User>>, StatusCode> {
    sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
