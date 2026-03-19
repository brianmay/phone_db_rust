use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, Queryable, Selectable};
use diesel_async::RunQueryDsl;

use chrono::DateTime;
use chrono::Utc;

use crate::models::defaults as model;
use crate::server::database::{connection::DatabaseConnection, schema};

#[allow(dead_code)]
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::defaults)]
pub struct Default {
    pub id: i64,
    pub order: Option<i32>,
    pub regexp: Option<String>,
    pub name: Option<String>,
    pub action: Option<String>,
    pub inserted_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Default> for model::Default {
    fn from(default: Default) -> Self {
        Self {
            id: model::DefaultId::new(default.id),
            order: default.order,
            regexp: default.regexp,
            name: default.name,
            action: default.action,
            inserted_at: default.inserted_at,
            updated_at: default.updated_at,
        }
    }
}

pub async fn search_defaults(
    conn: &mut DatabaseConnection,
    search: &str,
) -> Result<Vec<Default>, diesel::result::Error> {
    use crate::server::database::schema::defaults::dsl as q;
    use crate::server::database::schema::defaults::table;

    let search = search.replace("%", "\\%");

    table
        .select(Default::as_select())
        .filter(q::name.ilike(format!("%{}%", search)))
        .order((q::name.asc(),))
        .limit(10)
        .into_boxed()
        .get_results(conn)
        .await
}

pub async fn get_all_defaults(
    conn: &mut DatabaseConnection,
) -> Result<Vec<Default>, diesel::result::Error> {
    use crate::server::database::schema::defaults::dsl as q;
    use crate::server::database::schema::defaults::table;

    table
        .select(Default::as_select())
        .order((q::name.asc(),))
        .get_results(conn)
        .await
}

pub async fn get_default_by_id(
    conn: &mut DatabaseConnection,
    id: i64,
) -> Result<Option<Default>, diesel::result::Error> {
    use crate::server::database::schema::defaults as q;
    use crate::server::database::schema::defaults::table;

    table
        .select(Default::as_select())
        .filter(q::id.eq(id))
        .get_result(conn)
        .await
        .optional()
}

#[derive(Insertable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::defaults)]
pub struct NewDefault<'a> {
    pub order: Option<i32>,
    pub regexp: Option<&'a str>,
    pub name: Option<&'a str>,
    pub action: Option<&'a str>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'a> NewDefault<'a> {
    pub fn from_front_end(default: &'a model::NewDefault) -> Self {
        let now = chrono::Utc::now();
        Self {
            order: default.order,
            regexp: default.regexp.as_deref(),
            name: default.name.as_deref(),
            action: default.action.as_deref(),
            inserted_at: now,
            updated_at: now,
        }
    }
}

pub async fn create_default(
    conn: &mut DatabaseConnection,
    update: &NewDefault<'_>,
) -> Result<Default, diesel::result::Error> {
    use crate::server::database::schema::defaults::table;

    diesel::insert_into(table)
        .values(update)
        .returning(Default::as_returning())
        .get_result(conn)
        .await
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::defaults)]
pub struct ChangeDefault<'a> {
    pub order: Option<Option<i32>>,
    pub regexp: Option<Option<&'a str>>,
    pub name: Option<Option<&'a str>>,
    pub action: Option<Option<&'a str>>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl<'a> ChangeDefault<'a> {
    pub fn from_front_end(default: &'a model::ChangeDefault) -> Self {
        Self {
            order: default.order.into_option(),
            regexp: default.regexp.map_inner_deref().into_option(),
            name: default.name.map_inner_deref().into_option(),
            action: default.action.map_inner_deref().into_option(),
            inserted_at: None,
            updated_at: Some(Utc::now()),
        }
    }
}

pub async fn update_default(
    conn: &mut DatabaseConnection,
    id: i64,
    update: &ChangeDefault<'_>,
) -> Result<Default, diesel::result::Error> {
    use crate::server::database::schema::defaults::dsl as q;
    use crate::server::database::schema::defaults::table;

    diesel::update(table.filter(q::id.eq(id)))
        .set(update)
        .returning(Default::as_returning())
        .get_result(conn)
        .await
}

pub async fn delete_default(
    conn: &mut DatabaseConnection,
    id: i64,
) -> Result<(), diesel::result::Error> {
    use crate::server::database::schema::defaults::dsl as q;
    use crate::server::database::schema::defaults::table;

    diesel::delete(table.filter(q::id.eq(id)))
        .execute(conn)
        .await?;
    Ok(())
}
