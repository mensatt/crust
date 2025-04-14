pub mod db;
pub mod graphql;
pub mod schema;

use async_graphql::{dataloader::DataLoader, http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    http::Method,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::db::conn::get_db_pool;
use crate::graphql::dataloaders::*;
use crate::graphql::schema::*;

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Create dataloaders
    let location_loader = DataLoader::new(
        LocationLoader {
            pool: get_db_pool(),
        },
        tokio::spawn,
    );
    let dish_loader = DataLoader::new(
        DishLoader {
            pool: get_db_pool(),
        },
        tokio::spawn,
    );
    let tag_loader = DataLoader::new(
        TagLoader {
            pool: get_db_pool(),
        },
        tokio::spawn,
    );

    // Create GraphQL schema and add dataloaders and DB pool to its context
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(location_loader)
        .data(dish_loader)
        .data(tag_loader)
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
