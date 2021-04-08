use actix_web::{HttpResponse, Responder, delete, error::BlockingError, get, put, web};
use actix_multipart::Multipart;
use regex::Regex;
use crate::state::AppState;
use crate::{bucket, bucket::BucketError};
use super::response::*;

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
                Some(BucketError::NotFound) => HttpResponse::NotFound()
                    .json(Response::from(Error { message: format!("Not Found") })),
                _ => HttpResponse::InternalServerError()
                    .json(Response::from(Error { message: format!("Internal Server Error") })),
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
    data: web::Data<AppState>,
    request: web::HttpRequest
) -> impl Responder {
    // アップロードする際のフィールドの名前
    const FILE_FIELD_NAME: &'static str = "file";

    // 途中まで書いたやつ
    let re = Regex::new("x-amn-meta-(.+)").unwrap();
    let headers = request.headers();
    let _users_meta = headers.into_iter()
        .filter_map(|(name, value)| {
            re.captures(name.as_str())
                .and_then(|cap| cap.get(1))
                .map(|name| (
                    name.as_str().to_string(),
                    value.to_str().unwrap_or("").to_string()
                ))
        })
        .collect::<Vec<_>>();

    println!("{:?}", _users_meta);

    use futures::{StreamExt, TryStreamExt};
    while let Ok(Some(field)) = payload.try_next().await {
        let content_disposition = match field.content_disposition() {
            Some(c) => c,
            _ => continue,
        };

        // フィールド名が予期していないものは次のループに飛ばす
        if content_disposition
            .get_name()
            .map(|name| name != FILE_FIELD_NAME)
            .unwrap_or(false) {
            continue;
        }

        // チャンクを全てくっつけて一つの Vec<u8> にする
        let chunks = field
            .collect::<Vec<_>>()
            .await
            .iter()
            .flatten()
            .map(|c| c.to_vec())
            .flatten()
            .collect::<Vec<_>>();

        // ファイルを保存する
        return match web::block(move || bucket::put_file(data.data_directory.clone(), bucket_name, key, chunks)).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(_) => HttpResponse::InternalServerError().finish(),
        };
    }

    // ここに到達したということは、何もアップロードできていないということだと思うので...
    HttpResponse::BadRequest()
        .json(Response::from(Error { message: format!("Bad Request") }))
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