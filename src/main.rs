pub mod configuration;
pub mod router;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let host = "127.0.0.1:8080";
    println!("Started at: {}", host);
    let mut app = tide::new();
    router::setup(configuration::get_routes(), &mut app); 
    app.listen(host).await?;
    Ok(())
}
