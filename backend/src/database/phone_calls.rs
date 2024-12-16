use sqlx::postgres::PgPool;
use sqlx_conditional_queries::conditional_query_as;
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

pub async fn get_phone_calls(
    db: &PgPool,
    request: &PageRequest<PhoneCallKey>,
) -> Result<Page<PhoneCallDetails, PhoneCallKey>, sqlx::Error> {
    let search = request.search.as_ref().map(|s| format!("%{}%", s));
    let limit = 10;

    conditional_query_as!(
        PhoneCallDetails,
        r#"
        SELECT phone_calls.*, contacts.name as contact_name, contacts.phone_number as contact_phone_number, contacts.action as contact_action, contacts.comments as contact_comments, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
        FROM phone_calls
        INNER JOIN contacts ON contacts.id = phone_calls.contact_id
        {#where_clause}
        ORDER BY inserted_at, id DESC
        LIMIT {limit}
        "#,
        #where_clause = match (search, &request.after_key) {
            (Some(search), None) => "WHERE phone_number ILIKE {search} OR name ILIKE {search} OR destination_number ILIKE {search}",
            (None, Some(PhoneCallKey{inserted_at, id})) => "WHERE (phone_calls.inserted_at, phone_calls.id) > ({inserted_at},{id})",
            (Some(search), Some(PhoneCallKey{inserted_at, id})) => "WHERE (phone_number ILIKE {search} OR name ILIKE {search} OR destination_number ILIKE {search}) and (phone_calls.inserted_at, phone_calls.id) > ({inserted_at},{id})",
            (None, None) => "",
        }
    )
    .fetch_all(db)
    .await?
    .pipe(|x| list_to_page(x, limit))
    .pipe(Ok)
}
