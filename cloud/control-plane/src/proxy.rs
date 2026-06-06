use axum::body::Body;
use axum::http::{Request, StatusCode, Uri};
use axum::response::Response;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;

use crate::error::ApiError;

pub async fn forward_http(
    client: &Client,
    base_url: &str,
    engine_token: &str,
    method: axum::http::Method,
    path_and_query: &str,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, ApiError> {
    let url = format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        if path_and_query.starts_with('/') {
            path_and_query
        } else {
            "/"
        }
    );

    let bytes = axum::body::to_bytes(body, 16 * 1024 * 1024)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let mut req = client.request(method, &url).body(bytes);
    req = req.header(AUTHORIZATION, format!("Bearer {engine_token}"));

    for (name, value) in headers.iter() {
        if name == AUTHORIZATION {
            continue;
        }
        if let (Ok(hname), Ok(hval)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            req = req.header(hname, hval);
        }
    }

    let resp = req
        .send()
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut out_headers = HeaderMap::new();
    for (k, v) in resp.headers() {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(k.as_str().as_bytes()),
            HeaderValue::from_bytes(v.as_bytes()),
        ) {
            out_headers.insert(name, val);
        }
    }
    let body = resp
        .bytes()
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(Response::builder()
        .status(status)
        .body(Body::from(body))
        .unwrap())
}

pub fn join_proxy_path(tail: &str) -> String {
    if tail.is_empty() {
        "/".into()
    } else if tail.starts_with('/') {
        tail.to_string()
    } else {
        format!("/{tail}")
    }
}

pub fn uri_path_query(uri: &Uri) -> String {
    let path = uri.path();
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    format!("{path}{query}")
}
