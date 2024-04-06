#![allow(unused)]

mod models {
    pub mod post;
    pub mod user;
}
mod hashing;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};

mod db;
mod schema;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use models::{
    post::{NewPost, Post},
    user::{EmailPayload, User, UserUpdate, UserWithoutId},
};
use validator::Validate;

use std::{os::unix::net::SocketAddr, sync::Arc};

#[derive(Clone)]
struct AppState {
    connection: Pool<ConnectionManager<PgConnection>>,
}

#[tokio::main]
async fn main() {
    let data = db::get_connection_pool();
    let pool = data.clone();

    let shared_state = Arc::new(AppState { connection: pool });

    let routes = Router::new()
        .route(
            "/user",
            post(create_user)
                .put(update_user)
                .get(get_user)
                .delete(delete_user),
        )
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserWithoutId>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut connection = state.connection.get().unwrap();
    match User::create(payload, &mut connection) {
        Ok(message) => Ok(Html(message)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EmailPayload>,
) -> impl IntoResponse {
    let mut connection = state.connection.get().unwrap();
    match payload.validate() {
        Ok(_) => {}
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(e.to_string())
                .unwrap();
        }
    }

    match User::find_by_email(&payload.email, &mut connection) {
        Some(user) => {
            let user_json = serde_json::to_string(&user).unwrap();
            Response::builder()
                .status(StatusCode::OK)
                .body(user_json)
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("User not found".into())
            .unwrap(),
    }
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EmailPayload>,
) -> impl IntoResponse {
    let mut connection = state.connection.get().unwrap();

    match User::delete_by_email(&payload.email, &mut connection) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("User deleted"))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(e.to_string().into())
            .unwrap(),
    }
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserUpdate>,
) -> impl IntoResponse {
    let mut connection = state.connection.get().unwrap();

    match User::update_by_email(payload, &mut connection) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Updated User"))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(e.to_string().into())
            .unwrap(),
    }
}
