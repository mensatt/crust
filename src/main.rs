pub mod auth;
pub mod db;
pub mod graphql;
pub mod schema;

use std::sync::Arc;

use async_graphql::{dataloader::DataLoader, http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::{HeaderMap, Method},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::auth::{init_jwt_keypair, verify_jwt, AuthContext, JwtKeyPair};
use crate::db::conn::get_db_pool;
use crate::graphql::dataloaders::*;
use crate::graphql::schema::*;

// Embedd migrations into executable
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// State for Axum Router
#[derive(Clone)]
pub struct AppState {
    pub schema: GqlSchema,
    pub jwt_keypair: Arc<JwtKeyPair>,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Create database connection pool
    let db_pool = get_db_pool();

    // Run pending migrations
    // NOTE: We assume there already is database with the right name
    let mut conn = db_pool.get().expect("Failed to get connection from pool");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to apply pending migrations");

    // Read JWT keypair
    let jwt_keypair = Arc::new(init_jwt_keypair());

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

    // Create GraphQL schema with dataloaders, DB pool and JWT keypair in its context
    let schema: GqlSchema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(dish_loader)
        .data(location_loader)
        .data(occurrence_loader)
        .data(review_loader)
        .data(side_dish_loader)
        .data(tag_loader)
        .data(db_pool.clone())
        .data(jwt_keypair.clone())
        .finish();

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/playground", get(graphiql).post(graphql_handler))
        .with_state(AppState {
            schema,
            jwt_keypair,
        })
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

async fn graphql_handler(
    headers: HeaderMap,
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // (Try to) extract JWT claims from Bearer token in Authorization header
    let claims = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ")) // Extract Bearer value
        .and_then(|token| verify_jwt(token, &state.jwt_keypair.decoding_key).ok()); // Verify if it's a valid JWT

    println!("Request has claims: {:?}", claims);

    // Add the (optional) claims into the AuthContext and execute the given query
    state
        .schema
        .execute(req.into_inner().data(AuthContext { claims }))
        .await
        .into()
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
