use std::collections::HashMap;
use anyhow::Result;
use sha2::{Sha256, Digest};
use thiserror::Error;
use super::{Accessor, attributes::Attributes};

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
    accessor: Accessor<'a>,
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

    pub fn new(accessor: Accessor<'a>, directory: &'a str, bucket_name: &'a str) -> Bucket<'a> {
        Bucket { accessor, directory, bucket_name }
    }
}

// with self
impl <'a> Bucket<'a> {

    fn get_path(&self, key: &'a str) -> String {
        let bucket_name = Self::get_hex(&self.bucket_name);
        let key = Self::get_hex(&key);
        // とりあえずアンスコで繋げているが良いとは思えない...
        format!("{}_{}", bucket_name, key)
    }

    pub fn get_object(&self, key: &'a str) -> Result<Vec<u8>> {
        let path = self.get_path(key);
        let reader = self.accessor.get_reader(&path);
        let bytes = match reader.read() {
            Ok(bytes) => bytes,
            Err(_) => return Err(anyhow!(BucketError::NotFound)),
        };
        Ok(bytes)
    }

    /// オブジェクトを作成（更新）
    pub fn put_object(&self, key: &'a str, bytes: Vec<u8>) -> Result<()> {
        let path = self.get_path(key);
        let writer = self.accessor.get_writer(&path);
        if writer.write(bytes).is_err() {
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
        let writer = self.accessor.get_writer(&path);
        let _ = writer.remove()?; // 今考えんのめんどい
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
        let names = self.accessor.get_filenames()?
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
}
