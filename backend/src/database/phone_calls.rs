use sqlx::{
    postgres::{PgPool, PgRow},
    FromRow, QueryBuilder, Row,
};
use tap::Pipe;

use common::{Page, PageRequest, PhoneCallDetails, PhoneCallKey};

pub fn list_to_page(
    items: Vec<PhoneCallDetails>,
    limit: i64,
) -> Page<PhoneCallDetails, PhoneCallKey> {
    let length = i64::try_from(items.len()).unwrap_or(0);
    let last_key = items.last().map(|x| x.get_key());
    Page {
        items,
        next_key: if length >= limit { last_key } else { None },
    }
}

struct PhoneCallType(PhoneCallDetails);

impl FromRow<'_, PgRow> for PhoneCallType {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        PhoneCallDetails {
            inserted_at: row.try_get("inserted_at")?,
            updated_at: row.try_get("updated_at")?,
            id: row.try_get("id")?,
            contact_id: row.try_get("contact_id")?,
            // phone_number: row.try_get("phone_number")?,
            destination_number: row.try_get("destination_number")?,
            action: row.try_get::<String, _>("action")?.into(),
            contact_name: row.try_get("contact_name")?,
            contact_phone_number: row.try_get("contact_phone_number")?,
            contact_action: row.try_get::<String, _>("contact_action")?.into(),
            contact_comments: row.try_get("contact_comments")?,
            number_calls: row.try_get("number_calls")?,
        }
        .pipe(Self)
        .pipe(Ok)
    }
}

pub async fn get_phone_calls(
    db: &PgPool,
    request: &PageRequest<PhoneCallKey>,
    contact_id: Option<i64>,
) -> Result<Page<PhoneCallDetails, PhoneCallKey>, sqlx::Error> {
    let search = request.search.as_ref().map(|s| format!("%{}%", s));
    let limit = 10;

    let mut builder = QueryBuilder::new(
        r#"
            SELECT phone_calls.*, contacts.name as contact_name, contacts.phone_number as contact_phone_number, contacts.action as contact_action, contacts.comments as contact_comments, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
            FROM phone_calls
            INNER JOIN contacts ON contacts.id = phone_calls.contact_id
            "#,
    );

    let added_where = false;

    if let Some(id) = contact_id {
        if added_where {
            builder.push("AND ");
        } else {
            builder.push("WHERE ");
        }
        builder.push("contacts.id = ").push_bind(id);
    }

    if let Some(search) = search {
        if added_where {
            builder.push("AND ");
        } else {
            builder.push("WHERE ");
        }
        builder
            .push("phone_number ILIKE ")
            .push_bind(search.clone())
            .push(" OR name ILIKE ")
            .push_bind(search.clone())
            .push(" OR destination_number ILIKE ")
            .push_bind(search)
            .push(" ");
    }

    if let Some(PhoneCallKey { inserted_at, id }) = &request.after_key {
        if added_where {
            builder.push("AND ");
        } else {
            builder.push("WHERE ");
        }
        builder
            .push("(phone_calls.inserted_at, phone_calls.id) < (")
            .push_bind(inserted_at)
            .push(",")
            .push_bind(id)
            .push(") ");
    }

    builder
        .push("ORDER BY inserted_at DESC, id DESC ")
        .push("LIMIT ")
        .push_bind(limit);

    builder
        .build_query_as::<PhoneCallType>()
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|x| x.0)
        .collect::<Vec<_>>()
        .pipe(|x| list_to_page(x, limit))
        .pipe(Ok)
}
