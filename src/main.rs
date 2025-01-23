pub mod db;
pub mod schema;

use diesel::prelude::*;

use self::db::models::user::*;
use db::conn::establish_connection;

fn main() {
    use self::schema::users::dsl::*;

    let connection = &mut establish_connection();
    let results = users
        .limit(5)
        .select(User::as_select())
        .load(connection)
        .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
        println!(
            "{}, {} created at: {}",
            user.id, user.email, user.created_at
        );
    }
}
