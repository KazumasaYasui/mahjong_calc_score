use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use mahjong_core::{score as calc_score, ScoreRequest, ScoreResult};
use std::{net::SocketAddr, sync::Arc};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState {});
    let app = Router::new()
        .route("/", get(index))
        .route("/api/score", post(score))
        .nest_service("/static", ServeDir::new("mahjong_web/static"))
        .with_state(state);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("open http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn index() -> Response {
    (
        StatusCode::FOUND,
        [("Location", "/static/index.html")],
    )
        .into_response()
}

async fn score(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ScoreRequest>,
) -> Result<Json<ScoreResult>, ApiError> {
    let result = calc_score(&req);
    Ok(Json(result))
}

struct ApiError(String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.0).into_response()
    }
}
