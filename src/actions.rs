use diesel::prelude::*;
use uuid::Uuid;

use diesel_odbc::models;
use diesel_odbc::connection::RawConnection;
use odbc_safe as safe;

/// Run query using Diesel to find user by uid and return it.
pub fn find_user_by_uid<'env>(
    uid: Uuid,
    conn: &mut RawConnection<'env, safe::AutocommitOn>,
) -> Result<Option<models::User>, diesel::result::Error> {
    use diesel_odbc::schema::users::dsl::*;

    let user = users
        .filter(id.eq(uid.to_string()))
        .first::<models::User>(conn)
        .optional()?;

    Ok(user)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_user<'env>(
    // prevent collision with `name` column imported inside the function
    nm: &str,
    conn: &mut RawConnection<'env, safe::AutocommitOn>,
) -> Result<models::User, diesel::result::Error> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use diesel_odbc::schema::users::dsl::*;

    let new_user = models::User {
        id: Uuid::new_v4().to_string(),
        name: nm.to_owned(),
    };

    diesel::insert_into(users).values(&new_user).execute(conn)?;   

    Ok(new_user)
}
