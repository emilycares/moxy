use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use std::{convert::Infallible, net::SocketAddr};

pub mod builder;
pub mod configuration;
pub mod data_loader;
pub mod router;

extern crate pretty_env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    let mut config = configuration::get_configuration();

    if config.host == None {
        config.host = Some("127.0.0.1:8080".to_string());
    }

    let addr: Result<SocketAddr, _> = config.host.unwrap().parse();

    if let Ok(addr) = addr {
        let make_service =
            make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(router::endpoint)) });

        log::info!("Starting http server");
        let server = Server::bind(&addr).serve(make_service);

        // Run this server for... forever!
        if let Err(e) = server.await {
            log::error!("server error: {}", e);
        }
    } else {
        log::error!("Unable to start application with an invalid host");
    }

    Ok(())
}
