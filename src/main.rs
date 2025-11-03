pub mod auth;
pub mod config;
pub mod db;
pub mod graphql;
pub mod schema;
pub mod image_service;

use std::sync::Arc;

use anyhow::Context;
use async_graphql::{dataloader::DataLoader, http::GraphiQLSource, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLWebSocket, GraphQLProtocol};
use axum::{
    extract::{State, WebSocketUpgrade},
    http::{HeaderMap, Method},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::{debug, info};
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

use crate::auth::{init_jwt_keypair, verify_jwt, AuthContext, JwtKeyPair};
use crate::config::AppConfig;
use crate::db::conn::create_db_pool;
use crate::graphql::dataloaders::*;
use crate::graphql::schema::*;
use crate::graphql::subscriptions::SubscriptionBroker;
use crate::image_service::ImageServiceClient;

// Embed migrations into executable
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// State for Axum Router
#[derive(Clone)]
pub struct AppState {
    pub schema: GqlSchema,
    pub jwt_keypair: Arc<JwtKeyPair>,
    pub proxy_prefix: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load config
    let config = AppConfig::load()?;

    // Create JWT keypair
    let jwt_keypair = Arc::new(init_jwt_keypair(&config.jwt)?);

    // Create database connection pool
    let db_pool = create_db_pool(&config.database)?;

    // Run pending migrations
    // NOTE: We assume there already is database with the right name
    db_pool
        .get()
        .context("Unable to get connection from DB pool to run migrations")?
        .run_pending_migrations(MIGRATIONS)
        // Map to anyhow-compatible error
        .map_err(|e| anyhow::anyhow!(e))
        .context("Unable to apply migrations")?;

    info!("Pending migrations applied successfully");

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

    // Extract proxy prefix for Axum state
    let proxy_prefix = config.proxy_prefix.to_owned();

    // Create subscription broker
    let subscription_broker = SubscriptionBroker::new();

    let image_service_client = ImageServiceClient::new(config.image_service.clone());

    // Create GraphQL schema with dataloaders, DB pool, JWT keypair, config and subscription broker in its context
    let schema: GqlSchema = Schema::build(Query::default(), Mutation::default(), Subscription::default())
        .data(dish_loader)
        .data(location_loader)
        .data(occurrence_loader)
        .data(review_loader)
        .data(side_dish_loader)
        .data(tag_loader)
        .data(db_pool.clone())
        .data(jwt_keypair.clone())
        .data(subscription_broker.clone())
        .data(image_service_client)
        .data(config)
        .finish();

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/graphql", post(graphql_handler))
        .route("/graphql/ws", get(graphql_subscription_handler))
        .route("/playground", get(graphiql).post(graphql_handler))
        .with_state(AppState {
            schema,
            jwt_keypair,
            proxy_prefix: proxy_prefix.clone(),
        })
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|_, _| true))
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(AllowHeaders::any()),
        );

    // TODO: Make configurable
    const ADDR_AND_PORT: &str = "0.0.0.0:8000";
    let listener = tokio::net::TcpListener::bind(ADDR_AND_PORT)
        .await
        .context(format!(
            "Failed to create TcpListener for '{ADDR_AND_PORT}'"
        ))?;

    info!("Starting axum server on '{ADDR_AND_PORT}' with proxy prefix: '{proxy_prefix}'");
    axum::serve(listener, router.into_make_service())
        .await
        .context("Failed to start axum server")?;

    Ok(()) // This will likely never be reached
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

    if let Some(ref claims) = claims {
        debug!("Authenticated request with claims {:?}", claims);
    }

    // Add the (optional) claims into the AuthContext and execute the given query
    state
        .schema
        .execute(req.into_inner().data(AuthContext { claims }))
        .await
        .into()
}

async fn graphql_subscription_handler(
    State(state): State<AppState>,
    protocol: GraphQLProtocol,
    websocket: WebSocketUpgrade,
) -> impl IntoResponse {
    websocket.on_upgrade(move |stream| {
        GraphQLWebSocket::new(stream, state.schema, protocol)
            .serve()
    })
}

async fn graphiql(State(state): State<AppState>) -> impl IntoResponse {
    let endpoint = format!("{}/playground", state.proxy_prefix);
    let subscription_endpoint = format!("{}/graphql/ws", state.proxy_prefix);
    Html(
        GraphiQLSource::build()
            .endpoint(&endpoint)
            .subscription_endpoint(&subscription_endpoint)
            .finish(),
    )
}

async fn hello_world() -> &'static str {
    "Hello world from mensatt backend!"
}
