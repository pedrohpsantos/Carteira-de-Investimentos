use axum::{middleware, middleware::Next, extract::Request, response::Response, Router};
use sqlx::PgPool;
use tokio::net::TcpListener;
use std::time::Instant;
use tracing::info;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

use crate::routes;

async fn logging_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    let res = next.run(req).await;
    
    let latency = start.elapsed();
    let status = res.status();
    
    tracing::info!(
        "{} {} {} - {:?}",
        method,
        uri,
        status.as_u16(),
        latency
    );
    
    res
}

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

        // Start background worker
        crate::worker::start_price_simulator(crate::repository::Repository {
            db: state.db.clone(),
        })
        .await;

        let listener = TcpListener::bind("0.0.0.0:3000").await?;
        let router = Router::new()
            .nest("/api", routes::api::router())
            .merge(routes::frontend::router())
            .with_state(state)
            .layer(middleware::from_fn(logging_middleware));

        info!("Starting service");

        axum::serve(listener, router).await?;

        Ok(())
    }
}
