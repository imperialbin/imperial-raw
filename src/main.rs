mod database;
mod request_handler;

use std::convert::Infallible;
use std::env;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();

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
                request_handler::handle_request(req, pool.clone())
            }))
        }
    });

    // We'll bind to 127.0.0.1:3000
    let addr = ((
        [127, 0, 0, 1],
        env::var("PORT")
            .unwrap_or(String::from("3000"))
            .parse::<u16>()
            .unwrap(),
    ))
        .into();

    // Bind the server itself
    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", addr);

    // Wait for it to fail indefinetly
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
