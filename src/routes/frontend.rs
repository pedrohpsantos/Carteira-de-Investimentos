use askama::Template;
use axum::{
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    models::{Asset, PortfolioItem},
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
        .route("/portfolio", axum::routing::post(add_to_portfolio))
        .route("/assets/new", axum::routing::post(create_asset_frontend))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token))
        .http_only(true)
        .path("/")
        .secure(std::env::var("RENDER").is_ok()) // Only secure (HTTPS) on Render/Prod
        .same_site(axum_extra::extract::cookie::SameSite::Strict);

    Ok((jar.add(cookie), Redirect::to("/")))
}

async fn logout(jar: CookieJar) -> Result<impl IntoResponse, AppError> {
    let cookie = Cookie::build("token").path("/").build();
    Ok((jar.remove(cookie), Redirect::to("/login")))
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    user: User,
    portfolio: Vec<PortfolioItem>,
    assets: Vec<Asset>,
    total_value: f64,
}

async fn index(maybe_user: Option<User>, repository: Repository) -> Result<Response, AppError> {
    match maybe_user {
        Some(user) => {
            let portfolio = repository.list_portfolio(user.id()).await?;
            let assets = repository.list_assets().await?;
            let total_value: f64 = portfolio.iter().map(|item| item.total_value).sum();

            let page = DashboardPage {
                user,
                portfolio,
                assets,
                total_value,
            };
            Ok(Html(page.render()?).into_response())
        }
        None => Ok(Redirect::to("/login").into_response()),
    }
}

#[derive(Deserialize)]
struct PortfolioForm {
    asset_id: i64,
    quantity: f64,
}

async fn add_to_portfolio(
    user: User,
    repository: Repository,
    Form(form): Form<PortfolioForm>,
) -> Result<impl IntoResponse, AppError> {
    repository
        .add_to_portfolio(user.id(), form.asset_id, form.quantity)
        .await?;
    Ok(Redirect::to("/"))
}

#[derive(Deserialize)]
struct AssetForm {
    name: String,
    ticker: String,
    unit_value: f64,
}

async fn create_asset_frontend(
    _user: User,
    repository: Repository,
    Form(form): Form<AssetForm>,
) -> Result<impl IntoResponse, AppError> {
    repository
        .create_asset(form.name, form.ticker, form.unit_value)
        .await?;
    Ok(Redirect::to("/"))
}
