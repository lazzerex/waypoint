use toasty::Db;

pub async fn connect(database_url: &str) -> toasty::Result<Db> {
    Db::builder()
        .models(toasty::models!(crate::*))
        .connect(database_url)
        .await
}
