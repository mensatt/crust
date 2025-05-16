use async_graphql::{Context, Result, SimpleObject};
use diesel::prelude::*;

use crate::auth::AuthContext;
use crate::db::models::user::DbUser;
use crate::graphql::error::GqlApiError;
use crate::graphql::util::get_conn_from_ctx;

#[derive(Debug, SimpleObject)]
#[graphql(name = "User")]
pub struct GqlUser {
    pub id: uuid::Uuid,
    pub email: String,
}

impl From<DbUser> for GqlUser {
    fn from(value: DbUser) -> Self {
        GqlUser {
            id: value.id,
            email: value.email,
        }
    }
}

#[derive(Default)]
pub struct UserQueries;

#[async_graphql::Object]
impl UserQueries {
    async fn current_user(&self, ctx: &Context<'_>) -> Result<Option<GqlUser>> {
        use crate::schema::users::dsl::*;

        // Extract claims from auth context
        let claims = &ctx
            .data::<AuthContext>()
            .map_err(|e| {
                GqlApiError::internal("Unable to get AuthContext from context", e.message)
            })?
            .claims;

        // Query user based on sub claim (if present)
        match claims {
            Some(claims) => {
                // Get DB connection
                let conn = &mut get_conn_from_ctx(ctx)?;

                // Query user from DB based on sub claim
                let user = users
                    .filter(email.eq(&claims.sub))
                    .first::<DbUser>(conn)
                    .map_err(|e| {
                        // NOTE: NotFound could occur here (e.g. if user from JWT has been deleted
                        //       since creation of JWT), but we do not return it via the API, as
                        //       that would potentially expose information about the users table.
                        GqlApiError::internal(
                            format!("Failed to fetch user with email '{}'", claims.sub),
                            e.to_string(),
                        )
                    })?;

                Ok(Some(user.into()))
            }
            None => Ok(None),
        }
    }

    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<GqlUser>> {
        use crate::schema::users::dsl::*;

        // Require authentication for this query
        // This is done to avoid exposing users' emails
        ctx.data::<AuthContext>()?.require_auth()?;

        // Get DB connection
        let conn = &mut get_conn_from_ctx(ctx)?;

        let results = users
            .select(DbUser::as_select())
            .load(conn)
            .map_err(|e| GqlApiError::internal("Error while loading users", e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}
