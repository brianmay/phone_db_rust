use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, Queryable, Selectable};
use diesel_async::RunQueryDsl;

use chrono::DateTime;
use chrono::Utc;

use crate::models::contacts::ContactId;
use crate::models::phone_calls as model;
use crate::server::database::models::contacts::Contact;
use crate::server::database::{connection::DatabaseConnection, schema};

#[allow(dead_code)]
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::phone_calls)]
pub struct PhoneCall {
    pub id: i64,
    pub action: String,
    pub contact_id: i64,
    pub destination_number: Option<String>,
    pub inserted_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<PhoneCall> for model::PhoneCall {
    fn from(phone_call: PhoneCall) -> Self {
        Self {
            id: model::PhoneCallId::new(phone_call.id),
            action: phone_call.action,
            contact_id: ContactId::new(phone_call.contact_id),
            destination_number: phone_call.destination_number,
            inserted_at: phone_call.inserted_at,
            updated_at: phone_call.updated_at,
        }
    }
}

pub async fn search_phone_calls(
    conn: &mut DatabaseConnection,
    search: &str,
) -> Result<Vec<(PhoneCall, Contact)>, diesel::result::Error> {
    use crate::server::database::schema::contacts::dsl as c_q;
    use crate::server::database::schema::contacts::table as c_table;
    use crate::server::database::schema::phone_calls::dsl as q;
    use crate::server::database::schema::phone_calls::table;

    let search = search.replace("%", "\\%");

    table
        .inner_join(c_table)
        .select((PhoneCall::as_select(), Contact::as_select()))
        .filter(
            c_q::name
                .ilike(format!("%{}%", search))
                .or(c_q::phone_number.ilike(format!("%{}%", search))),
        )
        .order((q::inserted_at.desc(),))
        .limit(10)
        .into_boxed()
        .get_results(conn)
        .await
}

pub async fn get_phone_call_by_id(
    conn: &mut DatabaseConnection,
    id: i64,
) -> Result<Option<PhoneCall>, diesel::result::Error> {
    use crate::server::database::schema::phone_calls as q;
    use crate::server::database::schema::phone_calls::table;

    table
        .select(PhoneCall::as_select())
        .filter(q::id.eq(id))
        .get_result(conn)
        .await
        .optional()
}

#[derive(Insertable, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::phone_calls)]
pub struct NewPhoneCall<'a> {
    pub action: &'a str,
    pub contact_id: i64,
    pub destination_number: Option<&'a str>,
    pub inserted_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl<'a> NewPhoneCall<'a> {
    pub fn from_front_end(phone_call: &'a model::NewPhoneCall) -> Self {
        let now = chrono::Utc::now();
        Self {
            action: phone_call.action.as_ref(),
            contact_id: phone_call.contact_id.as_inner(),
            destination_number: phone_call.destination_number.as_deref(),
            inserted_at: now,
            updated_at: now,
        }
    }
}

pub async fn create_phone_call(
    conn: &mut DatabaseConnection,
    update: &NewPhoneCall<'_>,
) -> Result<PhoneCall, diesel::result::Error> {
    use crate::server::database::schema::phone_calls::table;

    diesel::insert_into(table)
        .values(update)
        .returning(PhoneCall::as_returning())
        .get_result(conn)
        .await
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::phone_calls)]
pub struct ChangePhoneCall<'a> {
    pub action: Option<&'a str>,
    pub contact_id: Option<i64>,
    pub destination_number: Option<Option<&'a str>>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl<'a> ChangePhoneCall<'a> {
    pub fn from_front_end(phone_call: &'a model::ChangePhoneCall) -> Self {
        Self {
            action: phone_call.action.as_deref().into_option(),
            contact_id: phone_call.contact_id.map(|x| x.as_inner()).into_option(),
            destination_number: phone_call
                .destination_number
                .map_inner_deref()
                .into_option(),
            inserted_at: None,
            updated_at: Some(Utc::now()),
        }
    }
}

pub async fn update_phone_call(
    conn: &mut DatabaseConnection,
    id: i64,
    update: &ChangePhoneCall<'_>,
) -> Result<PhoneCall, diesel::result::Error> {
    use crate::server::database::schema::phone_calls::dsl as q;
    use crate::server::database::schema::phone_calls::table;

    diesel::update(table.filter(q::id.eq(id)))
        .set(update)
        .returning(PhoneCall::as_returning())
        .get_result(conn)
        .await
}

pub async fn delete_phone_call(
    conn: &mut DatabaseConnection,
    id: i64,
) -> Result<(), diesel::result::Error> {
    use crate::server::database::schema::phone_calls::dsl as q;
    use crate::server::database::schema::phone_calls::table;

    diesel::delete(table.filter(q::id.eq(id)))
        .execute(conn)
        .await?;
    Ok(())
}
