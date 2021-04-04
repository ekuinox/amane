use actix_web::{Responder, get, put, web, HttpResponse};
use actix_multipart::Multipart;
use sha2::{Sha256, Digest};

// ファイルの保存先ディレクトリ
const TARGET_DIRECTORY: &'static str = "./tmp";

/// ファイルのパスを取得する
fn get_path(key: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sanitize_filename::sanitize(key));
    let key = hasher.finalize();
    let key = key.iter().map(|c| format!("{:02x}", c)).collect::<String>();
    format!("{}/{}", TARGET_DIRECTORY, key)
}

/// ファイルを取得するAPI
#[get("/{bucket_name}/{key:.*}")]
async fn get_file(web::Path((_bucket_name, key)): web::Path<(String, String)>) -> impl Responder {
    use std::io::Read;
    let filepath = get_path(&key);
    let file = match web::block(|| std::fs::File::open(filepath)).await {
        Ok(f) => f,
        Err(_) => return HttpResponse::NotFound().finish(),
    };
    let mut bytes: Vec<u8> = vec![];
    for byte in file.bytes() {
        match byte {
            Ok(byte) => bytes.push(byte),
            Err(_) => return HttpResponse::InternalServerError().finish(),
        }
    }
    HttpResponse::Ok().body(bytes)
}

/// ファイルを更新するAPI
#[put("/{bucket_name}/{key:.*}")]
async fn put_file(mut payload: Multipart, web::Path((_bucket_name, key)): web::Path<(String, String)>) -> impl Responder {
    use futures::{StreamExt, TryStreamExt};
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filepath = get_path(&key);

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
