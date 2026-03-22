use diesel_async::AsyncConnection;
use thiserror::Error;

use crate::models::defaults as models;
use crate::server::database::connection as database;
use crate::server::database::models::defaults;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] database::Error),
    #[error("Diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
}

pub async fn search_defaults(
    conn: &mut database::DatabaseConnection,
    query: String,
) -> Result<Vec<models::Default>, Error> {
    defaults::search_defaults(conn, &query)
        .await
        .map(|x| {
            x.into_iter()
                .map(|y| y.into())
                .collect::<Vec<models::Default>>()
        })
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_all_defaults(
    conn: &mut database::DatabaseConnection,
) -> Result<Vec<models::Default>, Error> {
    defaults::get_all_defaults(conn)
        .await
        .map(|x| {
            x.into_iter()
                .map(|y| y.into())
                .collect::<Vec<models::Default>>()
        })
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn get_default_by_id(
    conn: &mut database::DatabaseConnection,
    id: models::DefaultId,
) -> Result<Option<models::Default>, Error> {
    defaults::get_default_by_id(conn, id.as_inner())
        .await
        .map(|x| x.map(|y| y.into()))
        .map_err(database::Error::from)
        .map_err(Error::from)
}

pub async fn create_default(
    conn: &mut database::DatabaseConnection,
    default: models::NewDefault,
) -> Result<models::Default, Error> {
    let new_default = defaults::NewDefault::from_front_end(&default);

    conn.transaction::<_, Error, _>(move |conn| {
        let new_default = new_default.clone();
        Box::pin(async move {
            let default: models::Default = defaults::create_default(conn, new_default)
                .await
                .map(|x| x.into())
                .map_err(Error::from)?;

            Ok(default)
        })
    })
    .await
}

pub async fn update_default(
    conn: &mut database::DatabaseConnection,
    old_default: models::Default,
    change_default: models::ChangeDefault,
) -> Result<models::Default, Error> {
    let updates = defaults::ChangeDefault::from_front_end(&change_default);
    let old_default_id = old_default.id.as_inner();
    conn.transaction::<_, Error, _>(move |conn| {
        let updates = updates.clone();
        Box::pin(async move {
            let default: models::Default = defaults::update_default(conn, old_default_id, updates)
                .await
                .map(|x| x.into())
                .map_err(Error::from)?;

            Ok(default)
        })
    })
    .await
}

pub async fn delete_default(
    conn: &mut database::DatabaseConnection,
    old_default: models::Default,
) -> Result<(), Error> {
    let old_default_id = old_default.id.as_inner();

    conn.transaction::<_, Error, _>(move |conn| {
        Box::pin(async move {
            crate::server::database::models::defaults::delete_default(conn, old_default_id)
                .await
                .map_err(Error::from)?;

            Ok(())
        })
    })
    .await
}
