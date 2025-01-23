pub mod db;
pub mod graphql;
pub mod schema;

use crate::db::conn::get_db_pool;
use crate::graphql::schema::*;

use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    http::Method,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Initialize connection (pool) to DB
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(get_db_pool())
        .finish();

    let router = Router::new()
        .route("/", get(hello_world))
        .route(
            "/playground",
            get(graphiql).post_service(GraphQL::new(schema)),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|_, _| true))
                .allow_methods([Method::GET, Method::POST]),
        );

    let listener = tokio::net::TcpListener::bind("localhost:8000")
        .await
        .unwrap();

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/playground").finish())
}

async fn hello_world() -> &'static str {
    "Hello world from axum server!"
}
