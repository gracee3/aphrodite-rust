use aphrodite_api::config::Config;
use aphrodite_api::routes;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("aphrodite_api=info,tower_http=debug")),
        )
        .init();

    // Load configuration
    let config = Config::from_env();

    // Build application with middleware
    let app = routes::create_router()
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .into_inner(),
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting Aphrodite API server on {}", addr);

    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // In axum 0.7, Router should work directly - it implements IntoMakeService
    // If this doesn't compile, we may need to check axum version or use a workaround
    axum::serve(listener, app)
        .await
        .unwrap();
}

