use diesel;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct UserWithoutId {
    pub name: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}
#[derive(Deserialize, Debug, Validate)]
pub struct EmailPayload {
    #[validate(email)]
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Queryable, Selectable, Insertable, Validate)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Queryable, Serialize, Validate, Debug)]
pub struct UserWithoutPassword {
    pub id: Uuid,
    pub name: String,
    #[validate(email)]
    pub email: String,
}

impl User {
    pub fn new(user: UserWithoutId) -> Self {
        User {
            id: Uuid::new_v4(),
            name: user.name,
            email: user.email,
            password: user.password,
        }
    }

    pub fn create(user: UserWithoutId, conn: &mut PgConnection) -> Result<String, Box<dyn Error>> {
        use crate::schema::users;

        let new_user = User::new(user);

        match diesel::insert_into(users::table)
            .values(new_user)
            .returning(User::as_returning())
            .get_result(conn)
        {
            Ok(user) => {
                println!("User created: {:?}", user);
                Ok("User created".to_string())
            }
            Err(e) => {
                println!("Error creating user: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    pub fn find_by_email(email: &str, conn: &mut PgConnection) -> Option<UserWithoutPassword> {
        use crate::schema::users::dsl::*;

        let user: Option<UserWithoutPassword> = users
            .select((id, name, email))
            .filter(email.eq(email))
            .first::<UserWithoutPassword>(conn)
            .optional()
            .expect("Error loading user");
        user
    }

    pub fn delete_by_email(email: &str, conn: &mut PgConnection) -> Result<String, Box<dyn Error>> {
        use crate::schema::users::dsl::*;

        let user = diesel::delete(users.filter(email.eq(email))).execute(conn);

        match user {
            Ok(_) => Ok("User deleted".to_string()),
            Err(e) => Err(Box::new(e)),
        }
    }
}
