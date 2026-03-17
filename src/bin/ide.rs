/**
 * PLI Online IDE: serves the frontend and runs PLI code via the vm binary.
 */

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tower_http::services::ServeDir;

#[derive(serde::Deserialize)]
struct RunRequest {
    code: String,
    #[allow(dead_code)]
    input: Option<String>,
}

#[derive(serde::Serialize)]
struct RunResponse {
    stdout: String,
    stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn vm_binary_path() -> PathBuf {
    let exe = env::current_exe().expect("current_exe");
    let dir = exe.parent().expect("exe has parent");
    let name = format!("vm{}", env::consts::EXE_SUFFIX);
    dir.join(name)
}

async fn run_handler(Json(req): Json<RunRequest>) -> impl IntoResponse {
    let vm_path = vm_binary_path();
    if !vm_path.exists() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(RunResponse {
                stdout: String::new(),
                stderr: String::new(),
                error: Some(format!(
                    "vm binary not found at {}. Run: cargo build",
                    vm_path.display()
                )),
            }),
        );
    }

    let child = Command::new(&vm_path)
        .arg("--stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(RunResponse {
                    stdout: String::new(),
                    stderr: String::new(),
                    error: Some(format!("failed to spawn vm: {}", e)),
                }),
            );
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(req.code.as_bytes());
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(RunResponse {
                    stdout: String::new(),
                    stderr: String::new(),
                    error: Some(format!("failed to wait for vm: {}", e)),
                }),
            );
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    (
        StatusCode::OK,
        axum::Json(RunResponse {
            stdout,
            stderr,
            error: None,
        }),
    )
}

#[tokio::main]
async fn main() {
    let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    let app = Router::new()
        .route("/api/run", post(run_handler))
        .nest_service("/", ServeDir::new(static_dir));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("PLI IDE: http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
