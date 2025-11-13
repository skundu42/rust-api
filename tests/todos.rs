// Integration-style test that spins up the full router and issues requests as
// if we were an HTTP client. This gives new Rustaceans a practical example of
// how to exercise Axum handlers without opening a socket.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rust_api::{app, models::Todo, AppState};
use serde_json::json;
use tower::ServiceExt;

/// Walks through create -> read -> update -> list -> delete to demonstrate
/// the different status codes and payloads the service emits.
#[tokio::test]
async fn todo_crud_flow() {
    let state = AppState::new_in_memory();
    let app = app(state);

    let create_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/todos")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "title": "learn rust" }).to_string()))
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(create_res.status(), StatusCode::CREATED);
    let body = create_res
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let created: Todo = serde_json::from_slice(&body).unwrap();

    let get_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/todos/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(get_res.status(), StatusCode::OK);
    let fetched = serde_json::from_slice::<Todo>(
        &get_res
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();
    assert_eq!(fetched.title, created.title);

    let update_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/todos/{}", created.id))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "done": true }).to_string()))
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(update_res.status(), StatusCode::OK);
    let updated: Todo = serde_json::from_slice(
        &update_res
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes(),
    )
    .unwrap();
    assert!(updated.done);

    let list_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/todos")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(list_res.status(), StatusCode::OK);

    let delete_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/todos/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);

    let missing_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/todos/{}", created.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(missing_res.status(), StatusCode::NOT_FOUND);
}
