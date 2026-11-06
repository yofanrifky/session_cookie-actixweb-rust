use sqlx::PgPool;

pub async fn initialize_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    let statements = [
        "CREATE TABLE IF NOT EXISTS menus (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            price NUMERIC(12, 2) NOT NULL,
            category TEXT NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS orders (
            id SERIAL PRIMARY KEY,
            customer_name TEXT NOT NULL,
            item_name TEXT NOT NULL,
            quantity INT NOT NULL,
            status TEXT NOT NULL
        )",
        r#"CREATE TABLE IF NOT EXISTS "user" (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
        )"#,
    ];

    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }

    sqlx::query(r#"ALTER TABLE "user" ADD COLUMN IF NOT EXISTS password TEXT"#)
        .execute(pool)
        .await?;

    sqlx::query(
        "ALTER TABLE menus ALTER COLUMN price TYPE NUMERIC(12, 2) USING price::NUMERIC(12, 2)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
