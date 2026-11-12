use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use lets_do_it::UserDefined;
use serde_json::{json, Value};
use std::{
    path::Path,
    process::Stdio,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{process::Command, time::timeout};

const INDEX: &str = include_str!("../static/index.html");

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { Html(INDEX) }))
        .route("/analyze", post(analyze))
        .route("/run", post(run))
        .layer(DefaultBodyLimit::max(1_000_000));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    // 127.0.0.1 = only this computer can reach it
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    println!("listening on http://localhost:{port}");
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

async fn run(body: String) -> Response {
    let (hierarchy, parse_error): (Option<Value>, Option<String>) =
        match syn::parse_str::<syn::File>(&body) {
            Ok(ast) => (serde_json::to_value(UserDefined::new(&ast).hierarchy()).ok(), None),
            Err(e) => (None, Some(format!("Parse error: {e}"))),
        };

    let (run, run_error) = match run_locally(&body).await {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(e)),
    };

    Json(json!({
        "hierarchy": hierarchy,
        "parse_error": parse_error,
        "run": run,
        "run_error": run_error,
    }))
    .into_response()
}

// Writes the code to a temp folder, compiles it with rustc, runs it, cleans up.
async fn run_locally(code: &str) -> Result<Value, String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("lets_do_it_{}_{}", std::process::id(), nanos));
    tokio::fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;
    let result = compile_and_run(&dir, code).await;
    let _ = tokio::fs::remove_dir_all(&dir).await;
    result
}

async fn compile_and_run(dir: &Path, code: &str) -> Result<Value, String> {
    let src = dir.join("main.rs");
    let exe = dir.join(if cfg!(windows) { "main.exe" } else { "main" });
    tokio::fs::write(&src, code).await.map_err(|e| e.to_string())?;

    let compile = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output()
        .await
        .map_err(|e| format!("Could not start rustc (is it on your PATH?): {e}"))?;

    if !compile.status.success() {
        return Ok(json!({
            "success": false,
            "stdout": "",
            "stderr": String::from_utf8_lossy(&compile.stderr),
        }));
    }

    let child = Command::new(&exe)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true) // timeout drops the child, which kills it
        .spawn()
        .map_err(|e| e.to_string())?;

    match timeout(Duration::from_secs(10), child.wait_with_output()).await {
        Ok(Ok(out)) => Ok(json!({
            "success": out.status.success(),
            "stdout": String::from_utf8_lossy(&out.stdout),
            "stderr": String::from_utf8_lossy(&out.stderr),
        })),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("Timed out after 10 seconds (infinite loop?)".to_string()),
    }
}