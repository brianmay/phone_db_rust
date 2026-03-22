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

impl Contact {
    pub fn into_model(self, phone_call_count: i64) -> model::Contact {
        model::Contact {
            id: model::ContactId::new(self.id),
            phone_number: self.phone_number,
            name: self.name,
            action: self.action,
            comments: self.comments,
            phone_call_count,
            inserted_at: self.inserted_at,
            updated_at: self.updated_at,
        }
    }
}

/// Fetch the number of phone calls associated with a contact.
pub async fn get_phone_call_count(
    conn: &mut DatabaseConnection,
    contact_id: i64,
) -> Result<i64, diesel::result::Error> {
    use crate::server::database::schema::phone_calls::dsl as pc;
    use crate::server::database::schema::phone_calls::table as pc_table;
    use diesel::dsl::count_star;

    pc_table
        .filter(pc::contact_id.eq(contact_id))
        .select(count_star())
        .first(conn)
        .await
}

pub async fn search_contacts(
    conn: &mut DatabaseConnection,
    search: &str,
) -> Result<Vec<(Contact, i64)>, diesel::result::Error> {
    use crate::server::database::schema::contacts::dsl as q;
    use crate::server::database::schema::contacts::table;
    use crate::server::database::schema::phone_calls::dsl as pc;
    use crate::server::database::schema::phone_calls::table as pc_table;
    use diesel::dsl::count_star;

    let search = search.replace("%", "\\%");

    // Use a correlated subquery for the count so GROUP BY is not needed.
    let count_subquery = pc_table
        .filter(pc::contact_id.eq(q::id))
        .select(count_star())
        .single_value();

    table
        .select((Contact::as_select(), count_subquery))
        .filter(
            q::name
                .ilike(format!("%{}%", search))
                .or(q::phone_number.ilike(format!("%{}%", search))),
        )
        .order((q::name.asc(),))
        .limit(10)
        .into_boxed()
        .get_results::<(Contact, Option<i64>)>(conn)
        .await
        .map(|rows| rows.into_iter().map(|(c, n)| (c, n.unwrap_or(0))).collect())
}

pub async fn get_contact_by_id(
    conn: &mut DatabaseConnection,
    id: i64,
) -> Result<Option<(Contact, i64)>, diesel::result::Error> {
    use crate::server::database::schema::contacts as q;
    use crate::server::database::schema::contacts::table;

    let Some(contact) = table
        .select(Contact::as_select())
        .filter(q::id.eq(id))
        .get_result(conn)
        .await
        .optional()?
    else {
        return Ok(None);
    };

    let count = get_phone_call_count(conn, contact.id).await?;
    Ok(Some((contact, count)))
}

pub async fn get_contact_by_phone_number(
    conn: &mut DatabaseConnection,
    phone_number: &str,
) -> Result<Option<(Contact, i64)>, diesel::result::Error> {
    use crate::server::database::schema::contacts as q;
    use crate::server::database::schema::contacts::table;

    let Some(contact) = table
        .select(Contact::as_select())
        .filter(q::phone_number.eq(phone_number))
        .get_result(conn)
        .await
        .optional()?
    else {
        return Ok(None);
    };

    let count = get_phone_call_count(conn, contact.id).await?;
    Ok(Some((contact, count)))
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
