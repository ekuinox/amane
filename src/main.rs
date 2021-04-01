use std::io::Write;
use actix_web::{App, HttpResponse, Error, HttpServer, Responder, get, web};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};

#[get("/{id}/{name}")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}

async fn put_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        println!("{:?}", filename);
        let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }
    Ok(HttpResponse::Ok().into())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::new(|| App::new()
        .service(
            web::resource("/")
            .route(web::post().to(put_file))
        )
    )
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
