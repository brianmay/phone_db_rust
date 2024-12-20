use sqlx::postgres::PgPool;
use tap::Pipe;

use common::{Default, DefaultAddRequest, DefaultList, DefaultUpdateRequest};

pub async fn get_defaults(db: &PgPool) -> Result<DefaultList, sqlx::Error> {
    sqlx::query_as!(
        Default,
        r#"
        SELECT *
        FROM defaults
        ORDER BY defaults.order
        "#,
    )
    .fetch_all(db)
    .await?
    .pipe(DefaultList::new)
    .pipe(Ok)
}

pub async fn get_default(db: &PgPool, id: i64) -> Result<Default, sqlx::Error> {
    let result = sqlx::query_as!(
        Default,
        r#"
        SELECT *
        FROM defaults
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(db)
    .await?;

    Ok(result)
}

pub async fn update_default(
    db: &PgPool,
    request: DefaultUpdateRequest,
) -> Result<Default, sqlx::Error> {
    let time = chrono::Utc::now();
    let DefaultUpdateRequest {
        id,
        order,
        regexp,
        name,
        action,
    } = request;

    let result = sqlx::query!(
        r#"
        UPDATE defaults SET "order" = $2, regexp = $3, name = $4, action = $5, updated_at = $6
        WHERE id = $1
        "#,
        id,
        order,
        regexp,
        name,
        action.as_str(),
        time
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)
    } else {
        get_default(db, id).await
    }
}

pub async fn add_default(db: &PgPool, request: DefaultAddRequest) -> Result<Default, sqlx::Error> {
    let time = chrono::Utc::now();
    let DefaultAddRequest {
        order,
        regexp,
        name,
        action,
    } = request;

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO defaults ("order", regexp, name, action, inserted_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        order,
        regexp,
        name,
        action.as_str(),
        time,
        time
    )
    .fetch_one(db)
    .await?;

    Ok(Default {
        id,
        order,
        regexp,
        name,
        action,
        inserted_at: time,
        updated_at: time,
    })
}

pub async fn delete_default(db: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM defaults
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
