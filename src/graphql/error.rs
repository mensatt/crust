use std::panic::Location;

use async_graphql::Error;

/// Wrapper enum for various errors that might occur during GraphQL query/mutation execution.
/// This is used in (almost) all queries, mutations and resolvers to map occurring errors
/// to sensible GraphQL errors via .map_err(...)? syntax.
///
/// We (ab)use the conversion into async_graphql::Error to centralize logging for (most) errors.
/// NOTE: Authentication (incl. errors) is enforced via `require_auth` from src/auth.rs.
#[derive(Debug)]
pub enum GqlApiError {
    // Some internal error that we do not want to provide further context via GQL
    Internal {
        message: String,                      // Custom message to provide context
        location: &'static Location<'static>, // The location the error occurred at
        underlying_error: String,             // The underlying error (message)
    },
    // An error indicating that some requested entry was not not found
    NotFound(String),
}

impl From<GqlApiError> for async_graphql::Error {
    fn from(value: GqlApiError) -> Self {
        match value {
            GqlApiError::Internal {
                message,
                underlying_error,
                location,
            } => {
                log::error!(
                    "Internal error: '{}' (at {}:{}) | Underlying error: '{}'",
                    message,
                    location.file(),
                    location.line(),
                    underlying_error
                );
                // TODO: Does it make sense to return the "real" reason for authenticated users?
                Error::new("Encountered an internal error while processing this request. See server logs for more details.")
            }
            GqlApiError::NotFound(msg) => Error::new(msg),
        }
    }
}

// Small helper functions to make using the error variants easier
impl GqlApiError {
    #[track_caller]
    pub fn internal<S1: Into<String>, S2: Into<String>>(msg: S1, detail: S2) -> Self {
        let location = Location::caller();
        GqlApiError::Internal {
            message: msg.into(),
            underlying_error: detail.into(),
            location,
        }
    }

    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        GqlApiError::NotFound(msg.into())
    }
}
