use axum::{
    extract::{DefaultBodyLimit, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use lets_do_it::UserDefined;
use serde_json::{json, Value};
use std::time::Duration;

const INDEX: &str = include_str!("../static/index.html");

#[tokio::main]
async fn main() {
    let client = reqwest::Client::builder()
        .user_agent("lets-do-it-hierarchy")
        .build()
        .unwrap();

    let app = Router::new()
        .route("/", get(|| async { Html(INDEX) }))
        .route("/analyze", post(analyze))
        .route("/run", post(run))
        .layer(DefaultBodyLimit::max(200_000)) // 200 KB is plenty for pasted code
        .with_state(client);

    // Render sets PORT; 0.0.0.0 makes the server reachable from outside the container.
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    println!("listening on port {port}");
    axum::serve(listener, app).await.unwrap();
}

async fn analyze(body: String) -> Response {
    match syn::parse_str::<syn::File>(&body) {
        Ok(ast) => match UserDefined::new(&ast).to_json() {
            Ok(json) => ([(header::CONTENT_TYPE, "application/json")], json).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        Err(e) => (StatusCode::BAD_REQUEST, format!("Parse error: {e}")).into_response(),
    }
}

// Returns { hierarchy, parse_error, run, run_error }.
// The code is run by the Rust Playground's sandbox, never on this server.
async fn run(State(client): State<reqwest::Client>, body: String) -> Response {
    let (hierarchy, parse_error): (Option<Value>, Option<String>) =
        match syn::parse_str::<syn::File>(&body) {
            Ok(ast) => (serde_json::to_value(UserDefined::new(&ast).hierarchy()).ok(), None),
            Err(e) => (None, Some(format!("Parse error: {e}"))),
        };

    let req = json!({
        "channel": "stable",
        "mode": "debug",
        "edition": "2021",
        "crateType": "bin",
        "tests": false,
        "backtrace": false,
        "code": body,
    });

    let (run, run_error): (Option<Value>, Option<String>) = match client
        .post("https://play.rust-lang.org/execute")
        .timeout(Duration::from_secs(25))
        .json(&req)
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => match res.json::<Value>().await {
            Ok(v) => (
                Some(json!({
                    "success": v["success"],
                    "stdout": v["stdout"],
                    "stderr": v["stderr"],
                })),
                None,
            ),
            Err(e) => (None, Some(format!("Bad response from the runner: {e}"))),
        },
        Ok(res) => (None, Some(format!("The runner returned status {}", res.status()))),
        Err(e) => (None, Some(format!("Could not reach the runner: {e}"))),
    };

    Json(json!({
        "hierarchy": hierarchy,
        "parse_error": parse_error,
        "run": run,
        "run_error": run_error,
    }))
    .into_response()
}