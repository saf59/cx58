#![cfg(feature = "ssr")]

use axum::{routing::get, Router, middleware};
use axum::http::HeaderValue;
use axum::extract::Request;
use axum::middleware::Next;
use tower::ServiceExt;

async fn ok() -> &'static str { "ok" }

async fn security_headers(req: Request, next: Next) -> axum::response::Response {
    let mut res = next.run(req).await;
    let headers = res.headers_mut();

    let _ = headers.insert(
        axum::http::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    let _ = headers.insert(
        axum::http::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let _ = headers.insert(
        axum::http::HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );
    let csp = "default-src 'self'; frame-ancestors 'none'";
    if let Ok(val) = HeaderValue::from_str(csp) {
        let _ = headers.insert(axum::http::HeaderName::from_static("content-security-policy"), val);
    }
    res
}

#[tokio::test]
async fn sets_security_headers() {
    let app = Router::new()
        .route("/", get(ok))
        .layer(middleware::from_fn(security_headers));

    let req = axum::http::Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let h = resp.headers();
    assert_eq!(h.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(h.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(h.get("x-xss-protection").unwrap(), "1; mode=block");
    let csp = h.get("content-security-policy").unwrap().to_str().unwrap();
    assert!(csp.contains("default-src 'self'"));
}
