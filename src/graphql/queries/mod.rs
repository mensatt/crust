mod dish;
mod image;
mod location;
mod occurrence;
mod review;
mod tag;
mod user;

pub use dish::{DishQueries, GqlDish, GqlDishAlias, GqlReviewDataDish};
pub use image::{GqlImage, ImageQueries};
pub use location::{GqlLocation, LocationQueries};
pub use occurrence::{GqlOccurrence, GqlReviewDataOccurrence, OccurrenceQueries};
pub use review::{GqlReview, ReviewQueries};
pub use tag::{GqlTag, TagQueries};
pub use user::{GqlUser, UserQueries};
