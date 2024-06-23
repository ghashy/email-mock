use axum::extract::Path;
use axum::http::Method;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing;
use axum::Form;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let sigterm = signal(SignalKind::terminate())
        .expect("Failed to create signal handler");
    let subscriber = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::default())
        .with_max_level(tracing::Level::INFO)
        .compact()
        .with_level(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up tracing");

    println!("Starting email mock on 8001 port..");
    let listener = TcpListener::bind("0.0.0.0:8001").await.unwrap();
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/v1/smtp/:which", routing::post(get_email))
        .route("/v1/check/email/:email", routing::get(check_email))
        .route(
            "/healthcheck",
            routing::get(|| async {
                (StatusCode::OK, "Hello from email-mock!")
            }),
        )
        .layer(cors);
    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct EmailRequest {
    name: String,
    from: String,
    subject: String,
    to: String,
    html: String,
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Checks {
    pub is_disposable: bool,
    pub is_free_mail: bool,
    pub is_user_synonym: bool,
    pub valid_deliver: bool,
    pub valid_mx_record: bool,
    pub valid_syntax: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddressInfo {
    pub result: bool,
    pub source: String,
    pub email_address: String,
    pub has_user: String,
    pub has_real_user: String,
    pub has_domain: String,
    pub checks: Checks,
}

async fn get_email(
    Path(which): Path<String>,
    Form(request): Form<EmailRequest>,
) -> StatusCode {
    println!("Request: {}, method: {}", request.text, which);
    StatusCode::OK
}

async fn check_email(Path(email): Path<String>) -> Json<EmailAddressInfo> {
    println!("Requested email validation: {}", email);
    Json(EmailAddressInfo {
        result: true,
        source: email.clone(),
        email_address: email.clone(),
        has_user: "User".to_string(),
        has_real_user: "RealUser".to_string(),
        has_domain: "Domain".to_string(),
        checks: Checks {
            is_disposable: true,
            is_free_mail: true,
            is_user_synonym: true,
            valid_deliver: true,
            valid_mx_record: true,
            valid_syntax: true,
        },
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .expect("failed to install signal handler")
        .recv()
        .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("Terminate signal received");
}
