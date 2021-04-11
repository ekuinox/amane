mod attributes;

use anyhow::Result;
use attributes::Attributes;
use sha2::{Sha256, Digest};
use thiserror::Error;

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
    Ok(())
}

/// ファイルを保存する
pub fn put_file(directory: String, bucket_name: String, key: String, bytes: Vec<u8>) -> Result<()> {
    let filepath = get_path(&directory, &bucket_name, &key);
    let mut file = std::fs::File::create(filepath)?;
    use std::io::Write;
    if file.write_all(&bytes).is_err() {
        return Err(anyhow!(BucketError::Internal));
    }
    let paths = split_path(key.clone());
    for (name, child) in paths {
        let mut attr = Attributes::get_or_create(
            directory.clone(),
            bucket_name.clone(),
            name
        )?;
        let _ = attr.add_child(child.to_string())?;
        let _ = attr.save(directory.clone())?;
    }
    // とりあえずファイル作っとくだけ...
    let _ = Attributes::get_or_create(directory, bucket_name, key)?;
    Ok(())
}

/// `/` 区切りに 自身までのパスと子のパスに分解する
fn split_path(path: String) -> Vec<(String, String)> {
    let splited = path.split('/').collect::<Vec<_>>();
    let paths = splited.clone().into_iter().zip(splited.into_iter().skip(1)).collect::<Vec<_>>();
    let mut v = vec![];
    for (path, child) in paths {
        let name = v.last()
            .map(|(parent, _): &(String, String)| vec![parent.clone(), path.to_string()].join("/"))
            .unwrap_or(path.to_string());
        v.push((name, child.to_string()));
    }
    v
}
