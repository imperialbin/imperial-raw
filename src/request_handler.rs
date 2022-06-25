use std::convert::Infallible;
use std::time::Instant;

use super::http_util::{full_res, small_res, ResData};
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
    // get the basic data
    let method = req.method().to_owned();
    let path = req.uri().path().to_owned();

    // get now to measure time
    let now = Instant::now();

    // pas data to the normal handler
    match handle_request(req, pool).await {
        // if there was a response we log some data from it
        Ok(response) => {
            log::info!(
                "{} {} with {} in {}ms",
                method,
                path,
                response.status().as_u16(),
                now.elapsed().as_millis()
            );

            Ok(response)
        }
        // if there was an error we return 500
        Err(err) => {
            // capture the error and send to sentry
            // to analyze the error later
            sentry::capture_error(&err);

            log::warn!(
                "{} {} with 500 in {}ms",
                method,
                path,
                now.elapsed().as_millis()
            );

            Ok(small_res(500))
        }
    }
}

/// Main request handler
async fn handle_request(
    req: Request<Body>,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Response<Body>, RunError<tokio_postgres::Error>> {
    // extract method for later
    let method = req.method().to_owned();

    // only allow get
    if !check_method(method.clone()) {
        return Ok(small_res(405));
    }

    // if it's head it's prob CORS
    if method == Method::HEAD {
        return Ok(small_res(204));
    }

    // if path is empty return 404
    if req.uri() == "/" {
        return Ok(small_res(404));
    }

    // get id from path
    let id = &req.uri().path()[1..];

    // make the db request
    match super::database::make_db_req(id, pool).await? {
        // respond with plain text
        Some(res) => {
            return Ok(full_res(ResData {
                status: 200,
                body: Some(res),
                content_type: None,
            }))
        }
        // if res is none return 404
        None => return Ok(small_res(404)),
    };
}

fn check_method(method: Method) -> bool {
    match Some(method) {
        Some(Method::GET) => true,
        Some(Method::HEAD) => true,
        _ => false,
    }
}
