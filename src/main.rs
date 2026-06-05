mod auth;
mod handlers;
mod models;
mod psp;
mod state;
use axum::middleware;
use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};
use sqlx::migrate::Migrator;
use state::AppState;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    MIGRATOR
    .run(&pool)
    .await
    .expect("Failed to run migrations");

    let state = AppState { db: pool };

    let protected_routes = Router::new()
        .route(
            "/customers",
            post(handlers::create_customer).get(handlers::list_customers),
        )
        .route("/customers/{id}", get(handlers::get_customer))
        .route(
            "/invoices",
            post(handlers::create_invoice).get(handlers::list_invoices),
        )
        .route("/invoices/{id}", get(handlers::get_invoice))
        .route("/invoices/{id}/pay", post(handlers::pay_invoice))
        .route("/webhooks", post(handlers::create_webhook))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::api_key_auth,
        ));

    let app = Router::new()
        .route("/psp/charge", post(psp::charge))
        .route("/businesses", post(handlers::create_business))
        .merge(protected_routes)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server running on port 3000");

    axum::serve(listener, app).await.unwrap();
}
