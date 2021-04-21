use std::collections::HashMap;

use actix_web::{HttpResponse, Responder, delete, error::BlockingError, get, put, web};
use actix_multipart::Multipart;
use crate::{bucket::Accessor, state::AppState};
use crate::{bucket::Bucket, bucket::BucketError, bucket::is_users_meta_key};
use super::response::*;

/// ファイルを取得するAPI
#[get("/{bucket_name}/{key:.*}")]
async fn get_file(
    web::Path((bucket_name, key)): web::Path<(String, String)>,
    data: web::Data<AppState>
) -> impl Responder {
    match web::block(move || {
        let accessor = Accessor::new(&data.data_directory);
        let bucket = Bucket::new(
            accessor,
            &bucket_name
        );
        bucket.get_object(&key)
    }).await {
        Ok((bytes, attr)) => {
            let mut response = HttpResponse::Ok();
            // メタをそのままヘッダに付け加える
            for (key, value) in attr.meta() {
                response.set_header(key, value.clone());
            }
            response.body(bytes)
        },
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
    // key の最後の文字が / は許せません
    if key.chars().last() == Some('/') {
        return HttpResponse::BadRequest()
            .json(Response::from(Error { message: format!("Bad Request") }));
    }

    // アップロードする際のフィールドの名前
    const FILE_FIELD_NAME: &'static str = "file";

    // ヘッダからユーザ定義のメタ情報を取得する
    let headers = request.headers();
    let users_meta: HashMap<String, String> = headers.into_iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .filter(|(name, _)| is_users_meta_key(name.to_string()))
        .collect();

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
        return match web::block(move || {
            let accessor = Accessor::new(&data.data_directory);
            let bucket = Bucket::new(
                accessor,
                &bucket_name
            );
            let _ = bucket.put_object(&key, chunks)?;
            // TODO: これだとfileのデータなしではメタが更新できなくなってしまう
            bucket.update_meta(&key, users_meta)
        }).await {
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
    // メタも削除してやらなあかん...
    match web::block(move || {
        let accessor = Accessor::new(&data.data_directory);
        let bucket = Bucket::new(
            accessor,
            &bucket_name
        );
        bucket.delete_object(&key)
    }).await {
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

#[get("/search/{bucket_name}/{prefix:.*}")]
async fn search_files(
    web::Path((bucket_name, prefix)): web::Path<(String, String)>,
    data: web::Data<AppState>
) -> impl Responder {
    match web::block(move || {
        let accessor = Accessor::new(&data.data_directory);
        let bucket = Bucket::new(
            accessor,
            &bucket_name
        );
        bucket.list_objects(&prefix)
    }).await {
        Ok(files) => HttpResponse::Ok()
            .json(Response {
                ok: true,
                data: files,
            }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
