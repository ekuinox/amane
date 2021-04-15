use std::collections::HashMap;
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

#[derive(Clone, Debug)]
pub struct Bucket<'a> {
    // 保存先複数とかにできるようにはしたいから、このままは嫌かも~！
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

    /// prefix つっても read_dir でヒットするものしか拾えませ～ん
    fn get_filenames(prefix: &'a str) -> Result<Vec<String>> {
        let filenames = std::fs::read_dir(prefix)?
            .flatten()
            .map(|entry| entry.file_name().to_str().map(|name| name.to_string()))
            .flatten()
            .collect::<Vec<_>>();
        Ok(filenames)
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

    /// オブジェクトを作成（更新）
    pub fn put_object(&self, key: &'a str, bytes: Vec<u8>) -> Result<()> {
        let path = self.get_path(key);
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

    /// オブジェクトを削除
    pub fn delete_object(&self, key: &'a str) -> Result<()> {
        let path = self.get_path(key);
        if let Err(err) = std::fs::remove_file(path) {
            use std::io::ErrorKind;
            return Err(anyhow!(
                if err.kind() == ErrorKind::NotFound {
                    BucketError::NotFound
                } else {
                    BucketError::Internal
                }
            ));
        }
        let _ = Attributes::remove(
            self.directory.to_string(),
            self.bucket_name.to_string(),
            key.to_string()
        )?;
        Ok(())        
    }

    /// メタデータを更新
    pub fn update_meta(&self, key: &'a str, meta: HashMap<String, String>) -> Result<()> {
        let mut attr = Attributes::get_or_create(
            self.directory.to_string(),
            self.bucket_name.to_string(),
            key.to_string())?;
        for (k, v) in meta {
            attr.add_meta(k, v);
        }
        let _ = attr.save(self.directory.to_string())?;
        Ok(())
    }

    /// オブジェクトの一覧を取得
    pub fn list_objects(&self, prefix: &'a str) -> Result<Vec<String>> {
        let names = Self::get_filenames(&prefix)?
            .into_iter()
            .filter(|filename|
                filename.starts_with(Self::get_hex(&self.bucket_name).as_str())
            )
            .map(|name| format!("{}/{}", self.directory, name))
            .flat_map(|name| Attributes::from_filepath(name))
            .map(|attribute| attribute.name())
            .filter(|name| name.starts_with(prefix))
            .collect::<Vec<_>>();
        Ok(names)
    }

    /// すでにファイルが存在しているか
    #[allow(dead_code)]
    pub fn is_exists(&self, key: String) -> Result<bool> {
        let path = self.get_path(&key);
        match std::fs::File::open(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
