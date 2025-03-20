mod dish;
mod location;
mod tag;
mod user;

pub use dish::{DishQueries, GqlDish};
pub use location::{GqlLocation, LocationQueries};
pub use tag::{GqlTag, TagQueries};
pub use user::{GqlUser, UserQueries};
