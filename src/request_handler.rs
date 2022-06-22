use std::convert::Infallible;
use std::time::Instant;

use super::util::{full_res, small_res};
use bb8_postgres::bb8::Pool;
use bb8_postgres::bb8::RunError;
use bb8_postgres::tokio_postgres::{self, NoTls};
use bb8_postgres::PostgresConnectionManager;
use hyper::{Body, Method, Request, Response};

/// Catch all errors and a response
/// instead of closing the socket
pub async fn handle_with_errors(
    req: Request<Body>,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Response<Body>, Infallible> {
    // first get some basic data
    let basic: String = format!("{} {}", req.method(), req.uri().path());

    // get now to measure time
    let now = Instant::now();

    // pas data to the normal handler
    let response = handle_request(req, pool).await;

    match response {
        // if there was a response we log some data from it
        Ok(response) => {
            log::info!(
                "{} {} in {}ms",
                basic,
                response.status().as_u16(),
                now.elapsed().as_millis()
            );
            Ok(response)
        }
        // if there was an error we return 500
        Err(_) => {
            log::warn!("{} 500 in {}ms", basic, now.elapsed().as_millis());
            Ok(small_res(500))
        }
    }
}

/// Main request handler
async fn handle_request(
    req: Request<Body>,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Response<Body>, RunError<tokio_postgres::Error>> {
    // only allow get
    if req.method() != Method::GET {
        return Ok(small_res(405));
    }

    // if path is empty return 404
    if req.uri() == "/" {
        return Ok(small_res(404));
    }

    // get id from path
    let id = &req.uri().path()[1..];

    // make the db request
    let res = super::database::make_db_req(id, pool).await?;

    match res {
        // respond with plain text
        Some(res) => return Ok(full_res(200, None, Some(res.into()))),
        // if res is none return 404
        None => return Ok(small_res(404)),
    };
}
