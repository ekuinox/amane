mod routes;

use actix_web::{App, HttpServer};
use routes::*;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::new(|| App::new()
        .service(put_file)
        .service(get_file)
    )
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
