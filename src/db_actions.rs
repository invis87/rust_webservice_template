use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::data::*;
use crate::models;

pub fn insert_user(user_name: &str, conn: &PgConnection) -> Result<i32, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let new_user = models::NewUser { name: user_name };
    let result = diesel::insert_into(users)
        .values(&new_user)
        .get_result::<models::User>(conn)?;

    Ok(result.id)
}

pub fn get_user(
    user_id: i32,
    conn: &PgConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let result = users
        .filter(id.eq(user_id))
        .first::<models::User>(conn)
        .optional()?;

    Ok(result)
}
