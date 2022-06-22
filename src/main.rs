mod database;
mod logging;
mod request_handler;
mod util;

use std::convert::Infallible;
use std::env;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

#[tokio::main]
async fn main() -> Result<(), Infallible> {
    // init basics
    dotenv::dotenv().ok();
    logging::init_logger();

    // create a connection pool
    let pool = database::create_pool().await;

    // A `Service` is needed for every connection, so this
    // creates one from our `handle_request` and passes the
    // postgres client down
    let service = make_service_fn(move |_| {
        // clone the pool to pass it down
        let pool = pool.clone();
        // service_fn converts our function into a `Service`
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                request_handler::handle_with_errors(req, pool.clone())
            }))
        }
    });

    // get the port env or use 3000 as default
    let port = env::var("PORT")
        .unwrap_or(String::from("3000"))
        .parse::<u16>()
        .unwrap();

    // bind to 0.0.0.0:$PORT
    let addr = (([0, 0, 0, 0], port)).into();

    // Bind the server itself
    let server = Server::bind(&addr).serve(service);

    log::info!("Listening on http://localhost:{}", port);

    // Wait for it to fail indefinetly
    if let Err(e) = server.await {
        log::warn!("server error: {}", e);
    }

    Ok(())
}
