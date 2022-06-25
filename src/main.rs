mod database;
mod http_util;
mod logging;
mod request_handler;

use std::convert::Infallible;
use std::env;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

#[tokio::main]
async fn main() -> Result<(), Infallible> {
    // init basics
    dotenv::dotenv().ok();
    logging::init_logger();

    let _guard = sentry::init((
        env::var("SENTRY_DSN").unwrap(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some(env::var("RUST_ENV").unwrap_or("production".into()).into()),
            ..Default::default()
        },
    ));

    // create a connection pool
    let pool = database::create_pool().await;

    // A `Service` is needed for every connection, so this
    // creates one from our `handle_request` and passes the
    // postgres client down
    let service = make_service_fn(|_| {
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
        sentry::capture_error(&e);
    }

    Ok(())
}
