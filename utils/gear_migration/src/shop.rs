pub async fn items(url: &str) {
    let db_dynulo = bootstrap::DB::get().await;
    let db_synixe = bootstrap::DB::get_custom(url).await;
    let query =
        sqlx::query!("SELECT class, cost, roles, category, global FROM persistent_gear_items");
    let res = query.fetch_all(&*db_dynulo).await.unwrap();
    println!("Inserting {} items", res.len());
    for row in res {
        let (class, enabled) = if row.class.starts_with('$') {
            (row.class[1..].to_string(), false)
        } else {
            (row.class, true)
        };
        let roles = if let Some(roles) = row.roles {
            roles.split('|').map(|r| r.to_string()).collect()
        } else {
            vec![]
        };
        let query = sqlx::query(
            "INSERT INTO gear_items (class, roles, category, global, enabled) VALUES ($1, $2, $3, $4, $5)"
        )
            .bind(&class)
            .bind(roles)
            .bind(row.category)
            .bind(row.global)
            .bind(enabled);
        query.execute(&*db_synixe).await.unwrap();
        let query =
            sqlx::query("INSERT INTO gear_cost (class, cost, priority) VALUES ($1, $2, $3)");
        query
            .bind(class)
            .bind(row.cost)
            .bind(0)
            .execute(&*db_synixe)
            .await
            .unwrap();
    }
}
