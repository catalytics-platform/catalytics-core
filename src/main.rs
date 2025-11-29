use dotenvy::dotenv;
use tracing::info;
use catalytics_core::infrastructure::app::create_app;
use catalytics_core::infrastructure::setup::init_app_state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let app_state = init_app_state().await?;
    let app = create_app(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Catalytics Core listening at {}", &listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}