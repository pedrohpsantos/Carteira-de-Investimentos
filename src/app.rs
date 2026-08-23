use axum::Router;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    async fn new() -> color_eyre::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let db = PgPool::connect(&database_url).await?;

        // Run database migrations automatically on startup
        tracing::info!("Running database migrations...");
        sqlx::migrate!().run(&db).await?;
        tracing::info!("Database migrations applied.");

        Ok(Self { db })
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();

        tracing_subscriber::registry().with(layer).init();

        dotenvy::dotenv().ok();
        let state = AppState::new().await?;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .nest("/api", routes::api::router())
            .merge(routes::frontend::router())
            .with_state(state);

        info!("Starting service");

        axum::serve(listener, router).await?;

        Ok(())
    }
}
