mod attributes;

use std::collections::HashMap;

use anyhow::Result;
use attributes::Attributes;
use sha2::{Sha256, Digest};
use thiserror::Error;

pub use self::attributes::is_users_meta_key;

#[derive(Error, Debug)]
pub enum BucketError {
    #[error("file not found")]
    NotFound,
    #[error("internal error")]
    Internal,
}

/// 文字列をハッシュ化して16進数文字列にする
fn get_hex(value: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sanitize_filename::sanitize(value));
    let hashed = hasher.finalize();
    hashed.iter().map(|c| format!("{:02x}", c)).collect()
}

/// ファイルのパスを取得する
fn get_path(data_directory: &String, bucket_name: &String, key: &String) -> String {
    let bucket_name = get_hex(bucket_name);
    let key = get_hex(key);
    // とりあえずアンスコで繋げているが良いとは思えない...
    format!("{}/{}_{}", data_directory, bucket_name, key)
}

// ファイルをバイト配列として取得する
pub fn get_file_as_bytes(directory: String, bucket_name: String, key: String) -> Result<Vec<u8>> {
    use std::io::Read;
    let filepath = get_path(&directory, &bucket_name, &key);
    let mut file = match std::fs::File::open(filepath) {
        Ok(f) => f,
        Err(_) => return Err(anyhow!(BucketError::NotFound)),
    };
    let mut bytes = Vec::new();
    if let Err(_) = file.read_to_end(&mut bytes) {
        return Err(anyhow!(BucketError::Internal));
    }
    Ok(bytes)
}

/// ファイルを削除する
pub fn delete_file(directory: String, bucket_name: String, key: String) -> Result<()> {
    let filepath = get_path(&directory, &bucket_name, &key);
    if let Err(err) = std::fs::remove_file(filepath) {
        use std::io::ErrorKind;
        return Err(anyhow!(
            if err.kind() == ErrorKind::NotFound {
                BucketError::NotFound
            } else {
                BucketError::Internal
            }
        ));
    }
    let _ = Attributes::remove(directory, bucket_name, key)?;
    Ok(())
}

/// ファイルを保存する
/// 引数増えちゃったのどうにかしようね...
pub fn put_file(
    directory: String,
    bucket_name: String,
    key: String,
    bytes: Vec<u8>
) -> Result<()> {
    let filepath = get_path(&directory, &bucket_name, &key);
    let mut file = std::fs::File::create(filepath)?;
    use std::io::Write;
    if file.write_all(&bytes).is_err() {
        return Err(anyhow!(BucketError::Internal));
    }
    // とりあえずファイル作っとくだけ...
    let _ = Attributes::get_or_create(directory, bucket_name, key)?;
    Ok(())
}

pub fn update_meta(
    directory: String,
    bucket_name: String,
    key: String,
    meta: HashMap<String, String>
) -> Result<()> {
    let mut attr = Attributes::get_or_create(directory.clone(), bucket_name, key)?;
    for (k, v) in meta {
        attr.add_meta(k, v);
    }
    let _ = attr.save(directory)?;
    Ok(())
}

/// すでにファイルが存在しているか
#[allow(dead_code)]
pub fn is_exists(
    directory: String,
    bucket_name: String,
    key: String
) -> Result<bool> {
    let filepath = get_path(&directory, &bucket_name, &key);
    match std::fs::File::open(filepath) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(anyhow!(e)),
    }
}

/// prefix つっても read_dir でヒットするものしか拾えませ～ん
fn get_filenames<T: AsRef<str>>(prefix: T) -> Result<Vec<String>> {
    let filenames = std::fs::read_dir(prefix.as_ref().to_string())?
        .flatten()
        .map(|entry| entry.file_name().to_str().map(|name| name.to_string()))
        .flatten()
        .collect::<Vec<_>>();
    Ok(filenames)
}

/// prefix で始まるファイルを検索する
pub fn search_with_prefix(directory: String, bucket_name: String, prefix: String) -> Result<Vec<String>> {
    let names = get_filenames(&directory)?
        .into_iter()
        .filter(|filename| filename.starts_with(get_hex(&bucket_name).as_str()))
        .map(|name| format!("{}/{}", directory, name))
        .flat_map(|name| Attributes::from_filepath(name))
        .map(|attribute| attribute.name())
        .filter(|name| name.starts_with(prefix.as_str()))
        .collect::<Vec<_>>();
    Ok(names)
}
