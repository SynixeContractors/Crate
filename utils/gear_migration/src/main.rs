// mod bank;
mod shop;

#[tokio::main]
async fn main() {
    let db = std::env::var("DATABASE_URL").expect("Expected the DATABASE_URL in the environment");
    if db.contains("/synixe?") {
        panic!("This is meant to be run with the commander database in the environment")
    }
    let synixe_url = db.replace("/commander?", "/synixe?");

    // bank::deposits(&synixe_url).await;
    // bank::purchases(&synixe_url).await;
    // bank::transfers(&synixe_url).await;
    shop::items(&synixe_url).await;
}
