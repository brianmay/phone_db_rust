use sqlx::postgres::PgPool;
use sqlx_conditional_queries::conditional_query_as;
use tap::Pipe;

use common::{ContactDetails, ContactKey, ContactUpdateRequest, Page, PageRequest};

pub fn list_to_page(items: Vec<ContactDetails>, limit: i64) -> Page<ContactDetails, ContactKey> {
    let length = i64::try_from(items.len()).unwrap_or(0);
    let last_key = items.last().map(|x| x.get_key());
    Page {
        items,
        next_key: if length >= limit { last_key } else { None },
    }
}

pub async fn get_contacts(
    db: &PgPool,
    request: &PageRequest<ContactKey>,
) -> Result<Page<ContactDetails, ContactKey>, sqlx::Error> {
    let limit = 10;

    conditional_query_as!(
        ContactDetails,
        r#"
        SELECT *, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
        FROM contacts
        {#where_clause}
        ORDER BY phone_number, id
        LIMIT {limit}
        "#,
        #where_clause = match &request.after_key {
            Some(ContactKey{phone_number, id}) => "WHERE (phone_number, id) > ({phone_number},{id})",
            None => "",
        }
    )
    .fetch_all(db)
    .await?
    .pipe(|x| list_to_page(x, limit))
    .pipe(Ok)
}

pub async fn update_contact(
    db: &PgPool,
    request: &ContactUpdateRequest,
) -> Result<(), sqlx::Error> {
    let time = chrono::Utc::now();
    let ContactUpdateRequest {
        id,
        name,
        action,
        comments,
    } = request;

    let result = sqlx::query!(
        r#"
        UPDATE contacts SET name = $2, action = $3, comments = $4, updated_at = $5
        WHERE id = $1
        "#,
        id,
        *name,
        action.as_str(),
        *comments,
        time
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)?;
    }

    Ok(())
}
