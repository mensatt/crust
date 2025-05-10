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
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::db::conn::get_db_pool;
use crate::graphql::dataloaders::*;
use crate::graphql::schema::*;

// Embedd migrations into executable
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Create database connection pool
    let db_pool = get_db_pool();

    // Run pending migrations
    // NOTE: We assume there already is database with the right name
    let mut conn = db_pool.get().expect("Failed to get connection from pool");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to apply pending migrations");

    // Create dataloaders
    let dish_loader = DataLoader::new(
        DishLoader {
            pool: db_pool.clone(),
        },
        tokio::spawn,
    );
    let location_loader = DataLoader::new(
        LocationLoader {
            pool: db_pool.clone(),
        },
        tokio::spawn,
    );
    let occurrence_loader = DataLoader::new(
        OccurrenceLoader {
            pool: db_pool.clone(),
        },
        tokio::spawn,
    );
    let review_loader = DataLoader::new(
        ReviewLoader {
            pool: db_pool.clone(),
        },
        tokio::spawn,
    );
    let side_dish_loader = DataLoader::new(
        SideDishLoader {
            pool: db_pool.clone(),
        },
        tokio::spawn,
    );
    let tag_loader = DataLoader::new(
        TagLoader {
            pool: db_pool.clone(),
        },
        tokio::spawn,
    );

    // Create GraphQL schema and add dataloaders and DB pool to its context
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(dish_loader)
        .data(location_loader)
        .data(occurrence_loader)
        .data(review_loader)
        .data(side_dish_loader)
        .data(tag_loader)
        .data(db_pool.clone())
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/playground")
            .finish()
            // Replace lines were added because of
            //   https://github.com/async-graphql/async-graphql/issues/1703
            // they can be removed once the issue is resolved.
            .replace("@17", "@18")
            .replace(
                "ReactDOM.render(",
                "ReactDOM.createRoot(document.getElementById(\"graphiql\")).render(",
            ),
    )
}

async fn hello_world() -> &'static str {
    "Hello world from axum server!"
}
