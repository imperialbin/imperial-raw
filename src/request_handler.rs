use bb8_postgres::bb8::Pool;
use bb8_postgres::bb8::RunError;
use bb8_postgres::tokio_postgres::{self, NoTls};
use bb8_postgres::PostgresConnectionManager;
use hyper::{Body, Method, Request, Response, StatusCode};

pub async fn handle_request(
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
    let res = match make_db_req(id, pool).await {
        Ok(res) => res,
        // if it failed the call return 500
        Err(_) => return Ok(small_res(500)),
    };

    let res = match res {
        Some(res) => res,
        // if res is none return 404
        None => return Ok(small_res(404)),
    };

    // respond with plain text
    Ok(full_res(200, None, Some(res.into())))
}

async fn make_db_req(
    id: &str,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    // get a client from the pool
    let client = pool.get().await?;

    // prepare a statement
    let statement = client
        .prepare("SELECT content FROM documents WHERE id = $1")
        .await?;

    // query the database with the id
    let row = match client.query_opt(&statement, &[&id]).await? {
        Some(row) => row,
        None => return Ok(None),
    };

    

    Ok(Some(row.get::<&str, String>("content")))
}

fn small_res(status: u16) -> Response<Body> {
    full_res(status, None, None)
}

fn full_res(status: u16, content_type: Option<String>, body: Option<Body>) -> Response<Body> {
    // get status code obj
    let code = StatusCode::from_u16(status).unwrap();

    // get the correct content type
    let content_type = match content_type {
        Some(content_type) => content_type,
        None => String::from("text/plain"),
    };

    let body = match body {
        Some(body) => body,
        None => code.canonical_reason().unwrap().into(),
    };

    // build a pretty response
    Response::builder()
        .status(code)
        .header("access-control-allow-origin", "*")
        .header("content-type", content_type)
        .header("cache-control", "public, max-age=300")
        .body(body)
        .unwrap()
}
