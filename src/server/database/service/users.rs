use diesel_async::AsyncConnection;
use thiserror::Error;

use crate::models::users as models;
use crate::server::database::connection as database;
use crate::server::database::models::users;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] database::Error),
    #[error("Diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
}

pub async fn get_user_by_id(
    conn: &mut database::DatabaseConnection,
    id: models::UserId,
) -> Result<Option<models::User>, Error> {
    users::get_user_by_id(conn, id.as_inner())
        .await
        .map(|x| x.map(|y| y.into()))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_user_by_username(
    conn: &mut database::DatabaseConnection,
    username: &str,
) -> Result<Option<models::User>, Error> {
    users::get_user_by_username(conn, username)
        .await
        .map(|x| x.map(|y| y.into()))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_user_by_oidc_id(
    conn: &mut database::DatabaseConnection,
    oidc_id: &str,
) -> Result<Option<models::User>, Error> {
    users::get_user_by_oidc_id(conn, oidc_id)
        .await
        .map(|x| x.map(|y| y.into()))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_user_by_email(
    conn: &mut database::DatabaseConnection,
    email: &str,
) -> Result<Option<models::User>, Error> {
    users::get_user_by_email(conn, email)
        .await
        .map(|x| x.map(|y| y.into()))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_users(
    conn: &mut database::DatabaseConnection,
) -> Result<Vec<models::User>, Error> {
    users::get_users(conn)
        .await
        .map(|x| {
            x.into_iter()
                .map(|y| y.into())
                .collect::<Vec<models::User>>()
        })
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn create_user(
    conn: &mut database::DatabaseConnection,
    user: models::NewUser,
    hashed_password: &str,
) -> Result<models::User, Error> {
    let new_user = users::NewUser::from_front_end(&user, hashed_password);

    conn.transaction::<_, Error, _>(|conn| {
        Box::pin(async move {
            let user: models::User = users::create_user(conn, new_user)
                .await
                .map(|x| x.into())
                .map_err(Error::from)?;

            Ok(user)
        })
    })
    .await
}

pub async fn update_user(
    conn: &mut database::DatabaseConnection,
    old_user: models::User,
    change_user: models::ChangeUser,
    hashed_password: Option<String>,
) -> Result<models::User, Error> {
    let updates = users::UpdateUser::from_front_end(&change_user, hashed_password);

    conn.transaction::<_, Error, _>(|conn| {
        Box::pin(async move {
            let user: models::User = users::update_user(conn, old_user.id.as_inner(), updates)
                .await
                .map(|x| x.into())
                .map_err(Error::from)?;

            Ok(user)
        })
    })
    .await
}

pub async fn delete_user(
    conn: &mut database::DatabaseConnection,
    old_user: models::User,
) -> Result<(), Error> {
    conn.transaction::<_, Error, _>(|conn| {
        Box::pin(async move {
            crate::server::database::models::users::delete_user(conn, old_user.id.as_inner())
                .await
                .map_err(Error::from)?;

            Ok(())
        })
    })
    .await
}
