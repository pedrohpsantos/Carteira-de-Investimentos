use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{Asset, PortfolioItem, UserRecord},
};

pub struct Repository {
    pub db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as::<_, Asset>(
            "SELECT id, name, unit_value, ticker
             FROM assets;"
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(&self, name: String, ticker: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as::<_, Asset>(
            "INSERT INTO assets (name, ticker, unit_value)
             VALUES ($1, $2, $3)
             RETURNING id, name, unit_value, ticker;"
        )
        .bind(name)
        .bind(ticker)
        .bind(unit_value)
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        ticker: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as::<_, Asset>(
            "UPDATE assets
             SET name=COALESCE($2, name),
                 ticker=COALESCE($3, ticker),
                 unit_value=COALESCE($4, unit_value)
             WHERE id=$1
             RETURNING id, name, unit_value, ticker;"
        )
        .bind(asset_id)
        .bind(name)
        .bind(ticker)
        .bind(unit_value)
        .fetch_optional(&self.db)
        .await
    }

    pub async fn list_portfolio(&self, user_id: i64) -> sqlx::Result<Vec<PortfolioItem>> {
        sqlx::query_as::<_, PortfolioItem>(
            r#"SELECT 
                p.id, 
                p.asset_id, 
                a.name as asset_name, 
                a.ticker as asset_ticker,
                p.quantity, 
                a.unit_value,
                (p.quantity * a.unit_value) as total_value
             FROM portfolios p
             JOIN assets a ON p.asset_id = a.id
             WHERE p.user_id = $1;"#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
    }

    pub async fn add_to_portfolio(&self, user_id: i64, asset_id: i64, quantity: f64) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO portfolios (user_id, asset_id, quantity)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, asset_id) 
             DO UPDATE SET quantity = portfolios.quantity + $3;"
        )
        .bind(user_id)
        .bind(asset_id)
        .bind(quantity)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn remove_from_portfolio(&self, portfolio_id: i64, user_id: i64) -> sqlx::Result<()> {
        sqlx::query(
            "DELETE FROM portfolios WHERE id = $1 AND user_id = $2;"
        )
        .bind(portfolio_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as::<_, UserRecord>(
            "INSERT INTO users (username, password_hash)
             VALUES ($1, $2)
             RETURNING id, username, password_hash;"
        )
        .bind(username)
        .bind(password_hash)
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as::<_, UserRecord>(
            "SELECT id, username, password_hash
             FROM users
             WHERE username = $1;"
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use super::*;

    #[sqlx::test]
    async fn test_portfolio_operations(db: PgPool) {
        let repo = Repository::from(db);
        
        // Setup user and asset
        let user = repo.add_user("testuser", "hashed_pass").await.unwrap();
        let asset = repo.create_asset("Test Asset".to_string(), "TST".to_string(), 100.0).await.unwrap();
        
        // Add to portfolio
        repo.add_to_portfolio(user.id, asset.id, 5.0).await.unwrap();
        
        // List portfolio
        let portfolio = repo.list_portfolio(user.id).await.unwrap();
        assert_eq!(portfolio.len(), 1);
        assert_eq!(portfolio[0].asset_ticker, "TST");
        assert_eq!(portfolio[0].quantity, 5.0);
        assert_eq!(portfolio[0].total_value, 500.0);
        
        // Add more (upsert behavior)
        repo.add_to_portfolio(user.id, asset.id, 2.5).await.unwrap();
        let portfolio2 = repo.list_portfolio(user.id).await.unwrap();
        assert_eq!(portfolio2[0].quantity, 7.5);
        assert_eq!(portfolio2[0].total_value, 750.0);
    }
}
