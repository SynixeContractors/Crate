pub async fn deposits(url: &str) {
    let db_dynulo = bootstrap::DB::get().await;
    let db_synixe =
        bootstrap::DB::get_custom(url)
            .await;
    let query = sqlx::query!(
        "SELECT player, amount, reason, id, created FROM persistent_gear_bank_deposits"
    );
    let res = query.fetch_all(&*db_dynulo).await.unwrap();
    println!("Inserting {} deposits", res.len());
    for row in res {
        let query = sqlx::query(
            "INSERT INTO gear_bank_deposits (member, amount, reason, id, created) VALUES ($1, $2, $3, $4, $5)"
        )
            .bind(row.player)
            .bind(row.amount)
            .bind(row.reason)
            .bind(row.id)
            .bind(row.created);
        query.execute(&*db_synixe).await.unwrap();
    }
}

pub async fn purchases(url: &str) {
    let db_dynulo = bootstrap::DB::get().await;
    let db_synixe =
        bootstrap::DB::get_custom(url)
            .await;
    let query = sqlx::query!(
        "SELECT player, class, quantity, cost, global, created FROM persistent_gear_bank_purchases"
    );
    let res = query.fetch_all(&*db_dynulo).await.unwrap();
    println!("Inserting {} purchases", res.len());
    for row in res {
        let query = sqlx::query(
            "INSERT INTO gear_bank_purchases (member, class, quantity, cost, global, created) VALUES ($1, $2, $3, $4, $5, $6)"
        )
            .bind(row.player)
            .bind(row.class)
            .bind(row.quantity)
            .bind(row.cost)
            .bind(row.global)
            .bind(row.created);
        query.execute(&*db_synixe).await.unwrap();
    }
}

pub async fn transfers(url: &str) {
    let db_dynulo = bootstrap::DB::get().await;
    let db_synixe =
        bootstrap::DB::get_custom(url)
            .await;
    let query = sqlx::query!(
        "SELECT source, target, amount ,reason, created FROM persistent_gear_bank_transfers"
    );
    let res = query.fetch_all(&*db_dynulo).await.unwrap();
    println!("Inserting {} transfers", res.len());
    for row in res {
        let query = sqlx::query(
            "INSERT INTO gear_bank_transfers (source, target, amount, reason, created) VALUES ($1, $2, $3, $4, $5)"
        )
            .bind(row.source)
            .bind(row.target)
            .bind(row.amount)
            .bind(row.reason)
            .bind(row.created);
        query.execute(&*db_synixe).await.unwrap();
    }
}
