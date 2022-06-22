use hyper::{Body, Response, StatusCode};

pub fn small_res(status: u16) -> Response<Body> {
    full_res(status, None, None)
}

pub fn full_res(status: u16, content_type: Option<String>, body: Option<Body>) -> Response<Body> {
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
