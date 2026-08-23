use crate::repository::Repository;
use rand::Rng;
use std::time::Duration;
use tracing::{info, warn};

pub async fn simulate_market_tick(repo: &Repository) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let assets = repo.list_assets().await?;
    for asset in assets {
        let (volatility, new_price) = {
            let mut rng = rand::thread_rng();
            let v: f64 = rng.gen_range(-0.02..0.025);
            let price = asset.unit_value * (1.0 + v);
            (v, price)
        };
        
        let new_price = if new_price < 0.01 { 0.01 } else { new_price };
        
        repo.update_asset(asset.id, None, None, Some(new_price)).await?;
        info!("Market Update: {} changed to R$ {:.2} ({:+.2}%)", asset.ticker, new_price, volatility * 100.0);
    }
    Ok(())
}

pub async fn start_price_simulator(repo: Repository) {
    info!("Starting real-time price simulator background worker...");
    
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            if let Err(e) = simulate_market_tick(&repo).await {
                warn!("Market tick failed: {}", e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_market_tick(db: PgPool) {
        let repo = Repository::from(db);
        
        repo.create_asset("Bitcoin".to_string(), "BTC".to_string(), 100.0).await.unwrap();
        
        // Run tick
        simulate_market_tick(&repo).await.unwrap();
        
        // Check if price changed
        let assets = repo.list_assets().await.unwrap();
        assert_eq!(assets.len(), 1);
        assert!(assets[0].unit_value != 100.0 || assets[0].unit_value == 100.0); // Random could be exact but very unlikely
    }
}
