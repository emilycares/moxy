#[warn(missing_docs)]
pub mod builder;
#[warn(missing_docs)]
pub mod configuration;
#[warn(missing_docs)]
pub mod data_loader;
#[warn(missing_docs)]
pub mod router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    router::start().await;

    Ok(())
}
