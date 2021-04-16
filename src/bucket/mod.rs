mod attributes;
mod bucket;
mod accessor;

use sha2::{Sha256, Digest};
pub use self::bucket::{Bucket, BucketError};
pub use self::attributes::is_users_meta_key;

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
