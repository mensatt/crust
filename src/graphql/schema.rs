use crate::db::{
    conn::DbPool,
    models::{dish::*, location::*, tag::*, user::*},
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_graphql::Context;
use diesel::prelude::*;

#[derive(Default)]
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
        use crate::schema::users::dsl::*;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        let results = users
            .select(User::as_select())
            .load(conn)
            .expect("Error loading users");
        Ok(results)
    }

    async fn dishes(
        &self,
        ctx: &Context<'_>,
        filter: Option<DishFilter>,
    ) -> async_graphql::Result<Vec<Dish>> {
        use crate::schema::dishes;
        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct query
        let mut query = dishes::table.select(Dish::as_select()).into_boxed();

        // Add neccessary clauses depending on present filter values
        if let Some(f) = filter {
            if let Some(filter_dishes) = f.dishes {
                query = query.filter(dishes::id.eq_any(filter_dishes));
            }
            if let Some(filter_name_de) = f.name_de {
                query = query.filter(dishes::name_en.ilike(format!("%{}%", filter_name_de)));
            }
            if let Some(filter_name_en) = f.name_en {
                query = query.filter(dishes::name_en.ilike(format!("%{}%", filter_name_en)));
            }
        }

        // Return results
        let results = query.load(conn).expect("Error loading dishes");
        Ok(results)
    }

    async fn tags(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Tag>> {
        use crate::schema::tags::dsl::*;

        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        let query = tags.select(Tag::as_select());
        let results = query.load(conn).expect("Error loading tags");
        Ok(results)
    }

    async fn locations(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Location>> {
        use crate::schema::locations::dsl::*;

        // Get DB conn
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct and execute query
        let query = locations.select(Location::as_select());
        let results = query.load(conn).expect("Error loading tags");
        Ok(results)
    }
}

#[derive(Default)]
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> async_graphql::Result<User> {
        use crate::schema::users;
        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Hash password
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        let new_user = NewUser {
            id: uuid::Uuid::new_v4(),
            email: &email,
            password_hash: &password_hash,
            created_at: chrono::offset::Utc::now(),
            updated_at: chrono::offset::Utc::now(),
        };
        let results = diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
            .expect("Error saving new user");

        Ok(results)
    }

    async fn create_dish(
        &self,
        ctx: &Context<'_>,
        input: CreateDishInput,
    ) -> async_graphql::Result<Dish> {
        use crate::schema::dishes;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Construct new dish
        let new_dish = Dish {
            id: uuid::Uuid::new_v4(),
            name_de: input.name_de,
            name_en: input.name_en,
        };

        // Add dish
        let results = diesel::insert_into(dishes::table)
            .values(&new_dish)
            .get_result(conn)
            .expect("Error saving new dish");

        Ok(results)
    }

    async fn update_dish(
        &self,
        ctx: &Context<'_>,
        input: UpdateDishInput,
    ) -> async_graphql::Result<Dish> {
        use crate::schema::dishes;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Create query to update the given dish
        let query = diesel::update(dishes::table)
            .filter(dishes::id.eq(input.id))
            .set(&input);

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = query
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Get dish from DB if changeset was empty (== no changes should be made to object)
        let results = pot_empty_changeset.map(Ok).unwrap_or_else(|| {
            // Fallback query that returns the dish as it is stored in the databse
            dishes::table
                .filter(dishes::id.eq(input.id))
                .select(Dish::as_select())
                .first(conn)
        });

        Ok(results?)
    }

    async fn create_location(
        &self,
        ctx: &Context<'_>,
        input: CreateLocationInput,
    ) -> async_graphql::Result<Location> {
        use crate::schema::locations;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Add location
        let results = diesel::insert_into(locations::table)
            .values(&input)
            .get_result(conn)
            .expect("Error saving new tag");

        Ok(results)
    }

    async fn update_location(
        &self,
        ctx: &Context<'_>,
        input: UpdateLocationInput,
    ) -> async_graphql::Result<Location> {
        use crate::schema::locations;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Create query to update the given tag
        let query = diesel::update(locations::table)
            .filter(locations::id.eq(input.id))
            .set(&input);

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = query
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Get tag from DB if changeset was empty (== no changes should be made to object)
        let results = pot_empty_changeset.map(Ok).unwrap_or_else(|| {
            // Fallback query that returns the tag as it is stored in the databse
            locations::table
                .filter(locations::id.eq(input.id))
                .select(Location::as_select())
                .first(conn)
        });

        Ok(results?)
    }

    async fn create_tag(
        &self,
        ctx: &Context<'_>,
        input: CreateTagInput,
    ) -> async_graphql::Result<Tag> {
        use crate::schema::tags;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Add tag
        let results = diesel::insert_into(tags::table)
            .values(&input)
            .get_result(conn)
            .expect("Error saving new tag");

        Ok(results)
    }

    async fn update_tag(
        &self,
        ctx: &Context<'_>,
        input: UpdateTagInput,
    ) -> async_graphql::Result<Tag> {
        use crate::schema::tags;

        // Get DB connection
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut pool.get().unwrap();

        // Create query to update the given tag
        let query = diesel::update(tags::table)
            .filter(tags::key.eq(&input.key))
            .set(&input);

        // Try to update, map empty changeset to None (instead of Error)
        let pot_empty_changeset = query
            .get_result(conn)
            .optional_empty_changeset()
            .expect("Error while updating");

        // Get tag from DB if changeset was empty (== no changes should be made to object)
        let results = pot_empty_changeset.map(Ok).unwrap_or_else(|| {
            // Fallback query that returns the tag as it is stored in the databse
            tags::table
                .filter(tags::key.eq(&input.key))
                .select(Tag::as_select())
                .first(conn)
        });

        Ok(results?)
    }
}
