use sqlx::{
    postgres::{PgPool, PgRow},
    FromRow, QueryBuilder, Row,
};
use tap::Pipe;

use common::{
    ContactAddRequest, ContactDetails, ContactKey, ContactUpdateRequest, Page, PageRequest,
};

pub fn list_to_page(items: Vec<ContactDetails>, limit: i64) -> Page<ContactDetails, ContactKey> {
    let length = i64::try_from(items.len()).unwrap_or(0);
    let last_key = items.last().map(|x| x.get_key());
    Page {
        items,
        next_key: if length >= limit { last_key } else { None },
    }
}

struct ContactType(ContactDetails);

impl FromRow<'_, PgRow> for ContactType {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        ContactDetails {
            inserted_at: row.try_get("inserted_at")?,
            updated_at: row.try_get("updated_at")?,
            id: row.try_get("id")?,
            phone_number: row.try_get("phone_number")?,
            action: row.try_get::<String, _>("action")?.into(),
            name: row.try_get("name")?,
            comments: row.try_get("comments")?,
            number_calls: row.try_get("number_calls")?,
        }
        .pipe(Self)
        .pipe(Ok)
    }
}

pub async fn get_contacts(
    db: &PgPool,
    request: &PageRequest<ContactKey>,
) -> Result<Page<ContactDetails, ContactKey>, sqlx::Error> {
    let search = request.search.as_ref().map(|s| format!("%{}%", s));
    let limit = 10;

    let mut builder = QueryBuilder::new(
        r#"
            SELECT contacts.*, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
            FROM contacts
            "#,
    );

    let added_where = false;

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
            .push(" ");
    }

    if let Some(ContactKey { phone_number, id }) = &request.after_key {
        if added_where {
            builder.push("AND ");
        } else {
            builder.push("WHERE ");
        }
        builder
            .push("(contacts.phone_number, contacts.id) > (")
            .push_bind(phone_number)
            .push(",")
            .push_bind(id)
            .push(") ");
    }

    builder
        .push("ORDER BY inserted_at, id DESC ")
        .push("LIMIT ")
        .push_bind(limit);

    builder
        .build_query_as::<ContactType>()
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|x| x.0)
        .collect::<Vec<_>>()
        .pipe(|x| list_to_page(x, limit))
        .pipe(Ok)
}

pub async fn get_contact(db: &PgPool, id: i64) -> Result<ContactDetails, sqlx::Error> {
    let result = sqlx::query_as!(
        ContactDetails,
        r#"
        SELECT *, (SELECT COUNT(*) FROM phone_calls WHERE contact_id = contacts.id) as number_calls
        FROM contacts
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(db)
    .await?;

    Ok(result)
}

pub async fn update_contact(
    db: &PgPool,
    request: &ContactUpdateRequest,
) -> Result<(), sqlx::Error> {
    let time = chrono::Utc::now();
    let ContactUpdateRequest {
        id,
        phone_number,
        name,
        action,
        comments,
    } = request;

    let result = sqlx::query!(
        r#"
        UPDATE contacts SET phone_number = $2, name = $3, action = $4, comments = $5, updated_at = $6
        WHERE id = $1
        "#,
        id,
        *phone_number,
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

pub async fn add_contact(db: &PgPool, request: &ContactAddRequest) -> Result<(), sqlx::Error> {
    let time = chrono::Utc::now();
    let ContactAddRequest {
        phone_number,
        name,
        action,
        comments,
    } = request;

    let result = sqlx::query!(
        r#"
        INSERT INTO contacts (phone_number, name, action, comments, inserted_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        "#,
        *phone_number,
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

pub async fn delete_contact(db: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM contacts
        WHERE id = $1
        "#,
        id
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)?;
    }

    Ok(())
}
