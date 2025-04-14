mod dish;
mod location;
mod occurrence;
mod review;
mod tag;
mod user;

pub use dish::{DishQueries, GqlDish};
pub use location::{GqlLocation, LocationQueries};
pub use occurrence::{GqlOccurrence, OccurrenceQueries};
pub use review::{GqlReview, ReviewQueries};
pub use tag::{GqlTag, TagQueries};
pub use user::{GqlUser, UserQueries};
