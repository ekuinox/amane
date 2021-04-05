use actix_web::{HttpResponse, Responder, delete, error::BlockingError, get, put, web};
use actix_multipart::Multipart;
use crate::state::AppState;
use crate::{bucket, bucket::BucketError};

/// ファイルを取得するAPI
#[get("/{bucket_name}/{key:.*}")]
async fn get_file(
    web::Path((bucket_name, key)): web::Path<(String, String)>,
    data: web::Data<AppState>
) -> impl Responder {
    match web::block(move || bucket::get_file_as_bytes(data.data_directory.clone(), bucket_name, key)).await {
        Ok(bytes) => HttpResponse::Ok().body(bytes),
        Err(BlockingError::Error(err)) => {
            match err.downcast_ref::<BucketError>() {
                Some(BucketError::NotFound) => HttpResponse::NotFound().finish(),
                _ => HttpResponse::InternalServerError().finish(),
            }
        },
        _ => HttpResponse::InternalServerError().finish()
    }
}

/// ファイルを更新するAPI
#[put("/{bucket_name}/{key:.*}")]
async fn put_file(
    mut payload: Multipart,
    web::Path((bucket_name, key)): web::Path<(String, String)>,
    data: web::Data<AppState>
) -> impl Responder {
    // ここも bucket に良い具合にわけられたら良いんですが...
    // route から直接 bucket に行っているのも変な気はするが...
    use futures::{StreamExt, TryStreamExt};
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filepath = bucket::get_path(&data.data_directory, &bucket_name, &key);
        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            use std::io::Write;
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            match web::block(move || f.write_all(&data).map(|_| f)).await {
                Ok(a) => { f = a; },
                Err(_) => {
                    return HttpResponse::InternalServerError().finish();
                },
            }
        }
    }
    // 別に Ok とは限らないが...
    HttpResponse::Ok().finish()
}

#[delete("/{bucket_name}/{key:.*}")]
async fn delete_file(
    web::Path((bucket_name, key)): web::Path<(String, String)>,
    data: web::Data<AppState>
) -> impl Responder {
    match web::block(move || bucket::delete_file(data.data_directory.clone(), bucket_name, key)).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(BlockingError::Error(err)) => {
            match err.downcast_ref::<BucketError>() {
                Some(BucketError::NotFound) => HttpResponse::NotFound().finish(),
                _ => HttpResponse::InternalServerError().finish(),
            }
        },
        _ => HttpResponse::InternalServerError().finish(),
    }
}
