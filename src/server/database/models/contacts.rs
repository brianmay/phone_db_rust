use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, Queryable, Selectable};
use diesel_async::RunQueryDsl;

use chrono::DateTime;
use chrono::Utc;

use crate::models::contacts as model;
use crate::server::database::{connection::DatabaseConnection, schema};

#[allow(dead_code)]
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::contacts)]
pub struct Contact {
    pub id: i64,
    pub phone_number: String,
    pub name: Option<String>,
    pub action: String,
    pub comments: Option<String>,
    pub inserted_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Contact> for model::Contact {
    fn from(contact: Contact) -> Self {
        Self {
            id: model::ContactId::new(contact.id),
            phone_number: contact.phone_number,
            name: contact.name,
            action: contact.action,
            comments: contact.comments,
            inserted_at: contact.inserted_at,
            updated_at: contact.updated_at,
        }
    }
}

pub async fn search_contacts(
    conn: &mut DatabaseConnection,
    search: &str,
) -> Result<Vec<Contact>, diesel::result::Error> {
    use crate::server::database::schema::contacts::dsl as q;
    use crate::server::database::schema::contacts::table;

    let search = search.replace("%", "\\%");

    table
        .select(Contact::as_select())
        .filter(
            q::name
                .ilike(format!("%{}%", search))
                .or(q::phone_number.ilike(format!("%{}%", search))),
        )
        .order((q::name.asc(),))
        .limit(10)
        .into_boxed()
        .get_results(conn)
        .await
}

pub async fn get_contact_by_id(
    conn: &mut DatabaseConnection,
    id: i64,
) -> Result<Option<Contact>, diesel::result::Error> {
    use crate::server::database::schema::contacts as q;
    use crate::server::database::schema::contacts::table;

    table
        .select(Contact::as_select())
        .filter(q::id.eq(id))
        .get_result(conn)
        .await
        .optional()
}

pub async fn get_contact_by_phone_number(
    conn: &mut DatabaseConnection,
    phone_number: &str,
) -> Result<Option<Contact>, diesel::result::Error> {
    use crate::server::database::schema::contacts as q;
    use crate::server::database::schema::contacts::table;

    table
        .select(Contact::as_select())
        .filter(q::phone_number.eq(phone_number))
        .get_result(conn)
        .await
        .optional()
}

#[derive(Insertable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::contacts)]
pub struct NewContact<'a> {
    pub phone_number: &'a str,
    pub name: Option<&'a str>,
    pub action: &'a str,
    pub comments: Option<&'a str>,
    pub inserted_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl<'a> NewContact<'a> {
    pub fn from_front_end(contact: &'a model::NewContact) -> Self {
        let now = chrono::Utc::now();
        Self {
            phone_number: &contact.phone_number,
            name: contact.name.as_deref(),
            action: &contact.action,
            comments: contact.comments.as_deref(),
            inserted_at: now,
            updated_at: now,
        }
    }
}

pub async fn create_contact(
    conn: &mut DatabaseConnection,
    update: &NewContact<'_>,
) -> Result<Contact, diesel::result::Error> {
    use crate::server::database::schema::contacts::table;

    diesel::insert_into(table)
        .values(update)
        .returning(Contact::as_returning())
        .get_result(conn)
        .await
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::contacts)]
pub struct ChangeContact<'a> {
    pub phone_number: Option<&'a str>,
    pub name: Option<Option<&'a str>>,
    pub action: Option<&'a str>,
    pub comments: Option<Option<&'a str>>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl<'a> ChangeContact<'a> {
    pub fn from_front_end(contact: &'a model::ChangeContact) -> Self {
        Self {
            phone_number: contact.phone_number.as_deref().into_option(),
            name: contact.name.map_inner_deref().into_option(),
            action: contact.action.as_deref().into_option(),
            comments: contact.comments.map_inner_deref().into_option(),
            inserted_at: None,
            updated_at: Some(Utc::now()),
        }
    }
}

pub async fn update_contact(
    conn: &mut DatabaseConnection,
    id: i64,
    update: &ChangeContact<'_>,
) -> Result<Contact, diesel::result::Error> {
    use crate::server::database::schema::contacts::dsl as q;
    use crate::server::database::schema::contacts::table;

    diesel::update(table.filter(q::id.eq(id)))
        .set(update)
        .returning(Contact::as_returning())
        .get_result(conn)
        .await
}

pub async fn delete_contact(
    conn: &mut DatabaseConnection,
    id: i64,
) -> Result<(), diesel::result::Error> {
    use crate::server::database::schema::contacts::dsl as q;
    use crate::server::database::schema::contacts::table;

    diesel::delete(table.filter(q::id.eq(id)))
        .execute(conn)
        .await?;
    Ok(())
}
