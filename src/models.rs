use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, Clone, FromRow)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
    pub ticker: String,
}

#[derive(FromRow)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Clone, FromRow)]
pub struct PortfolioItem {
    pub id: i64,
    pub asset_id: i64,
    pub asset_name: String,
    pub asset_ticker: String,
    pub quantity: f64,
    pub unit_value: f64,
    pub total_value: f64,
}
