use anyhow::Result;
use sha2::{Sha256, Digest};
use thiserror::Error;
use super::attributes::Attributes;

#[derive(Error, Debug)]
pub enum BucketError {
    #[error("file not found")]
    NotFound,
    #[error("internal error")]
    Internal,
}

pub struct Bucket<'a> {
    directory: &'a str,
    bucket_name: &'a str,
}

// without self
impl <'a> Bucket<'a> {
    /// 文字列をハッシュ化して16進数文字列にする
    fn get_hex(value: &'a str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(sanitize_filename::sanitize(value));
        let hashed = hasher.finalize();
        hashed.iter().map(|c| format!("{:02x}", c)).collect()
    }

    pub fn new(directory: &'a str, bucket_name: &'a str) -> Bucket<'a> {
        Bucket { directory, bucket_name }
    }
}

// with self
impl <'a> Bucket<'a> {

    fn get_path(&self, key: &'a str) -> String {
        let bucket_name = Self::get_hex(&self.bucket_name);
        let key = Self::get_hex(&key);
        // とりあえずアンスコで繋げているが良いとは思えない...
        format!("{}/{}_{}", self.directory, bucket_name, key)
    }

    pub fn get_object(&self, key: &'a str) -> Result<Vec<u8>> {
        use std::io::Read;
        let path = self.get_path(key);
        let mut file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return Err(anyhow!(BucketError::NotFound)),
        };
        let mut bytes = Vec::new();
        if let Err(_) = file.read_to_end(&mut bytes) {
            return Err(anyhow!(BucketError::Internal));
        }
        Ok(bytes)
    }

    pub fn put_object(&self, key: &'a str, bytes: Vec<u8>) -> Result<()> {
        let a = key.as_ref();
        let path = self.get_path(a);
        let mut file = std::fs::File::create(path)?;
        use std::io::Write;
        if file.write_all(&bytes).is_err() {
            return Err(anyhow!(BucketError::Internal));
        }
        // とりあえずファイル作っとくだけ...
        let _ = Attributes::get_or_create(
            self.directory.to_string(),
            self.bucket_name.to_string(),
            key.to_string()
        )?;
        
        Ok(())
    }
}
