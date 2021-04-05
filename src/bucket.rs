use anyhow::Result;
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
/// pub を外したい
pub fn get_path(data_directory: &String, bucket_name: &String, key: &String) -> String {
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
