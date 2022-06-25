use etag::EntityTag;
use httpdate::fmt_http_date;
use hyper::header;
use hyper::{Body, Response, StatusCode};
use std::time::{Duration, SystemTime};

pub struct ResData {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Option<String>,
}

pub fn small_res(status: u16) -> Response<Body> {
    full_res(ResData {
        status,
        body: None,
        content_type: None,
    })
}

pub fn full_res(data: ResData) -> Response<Body> {
    // get status code obj
    let code = StatusCode::from_u16(data.status).unwrap();

    // get the correct content type
    let content_type = match data.content_type {
        Some(content_type) => content_type,
        None => String::from("text/plain"),
    };

    // default body
    let body = match data.body {
        Some(body) => body,
        None => code.canonical_reason().unwrap().to_string(),
    };

    // expires header
    let expire: u64 = 300;

    // build a pretty response
    let builder = Response::builder()
        .status(code)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_MAX_AGE, format!("{}", expire))
        .header(header::CACHE_CONTROL, format!("public, max-age={}", expire))
        .header(header::VARY, header::ORIGIN)
        .header(
            header::EXPIRES,
            fmt_http_date(SystemTime::now() + Duration::new(expire, 0)),
        );

    // if status is no content make the
    // remove body and some headers etc
    let response = if data.status != 204 {
        builder
            .header(header::CONTENT_TYPE, content_type)
            .header(header::ETAG, EntityTag::from_data(body.as_bytes()).tag())
            .body(body.into())
    } else {
        builder.body("".into())
    };

    response.unwrap()
}
