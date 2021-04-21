use std::{collections::HashMap, convert::{TryFrom, TryInto}};
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

    pub fn new(accessor: Accessor<'a>, bucket_name: &'a str) -> Bucket<'a> {
        Bucket { accessor, bucket_name }
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

    pub fn get_object(&self, key: &'a str) -> Result<(Vec<u8>, Attributes)> {
        let path = self.get_path(key);
        let reader = self.accessor.get_reader(&path);
        let bytes = match reader.read() {
            Ok(bytes) => bytes,
            Err(_) => return Err(anyhow!(BucketError::NotFound)),
        };
        let attr_path = Attributes::get_path(&path);
        let reader = self.accessor.get_reader(&attr_path);
        let attr: Attributes = match reader.read() {
            Ok(bytes) => bytes.try_into()?,
            Err(_) => return Err(anyhow!(BucketError::Internal)),
        };
        Ok((bytes, attr))
    }

    /// オブジェクトを作成（更新）
    pub fn put_object(&self, key: &'a str, bytes: Vec<u8>) -> Result<()> {
        // オブジェクト本体の保存
        let path = self.get_path(key);
        let writer = self.accessor.get_writer(&path);
        let _ = writer.write(bytes)?;

        // 付属情報の保存
        // 新規になってしまっているのでやばいかも！
        let attr = Attributes::new(self.bucket_name.to_string(), key.to_string());
        let attr_path = Attributes::get_path(&path);
        let bytes: Vec<u8> = attr.try_into()?;
        let writer = self.accessor.get_writer(&attr_path);
        let _ = writer.write(bytes)?;

        Ok(())
    }

    /// オブジェクトを削除
    pub fn delete_object(&self, key: &'a str) -> Result<()> {
        // オブジェクトの削除
        let path = self.get_path(key);
        let writer = self.accessor.get_writer(&path);
        let _ = writer.remove()?; // 今考えんのめんどい

        // 付属情報の削除
        let attr_path = Attributes::get_path(&path);
        let writer = self.accessor.get_writer(&attr_path);
        let _ = writer.remove()?;

        Ok(())        
    }

    /// メタデータを更新
    pub fn update_meta(&self, key: &'a str, meta: HashMap<String, String>) -> Result<()> {
        let path = self.get_path(key);
        let attr_path = Attributes::get_path(&path);
        let reader = self.accessor.get_reader(&attr_path);
        let mut attr = match reader.read() {
            Ok(bytes) => Attributes::try_from(bytes)?, // 取れないとか意味わからん
            Err(_) => Attributes::new(self.bucket_name.to_string(), key.to_string()),
        };
        for (k, v) in meta {
            attr.add_meta(k, v);
        }
        
        // 変更したものを書き込む
        let bytes: Vec<u8> = attr.try_into()?;
        let writer = self.accessor.get_writer(&attr_path);
        let _ = writer.write(bytes)?;

        Ok(())
    }

    /// オブジェクトの一覧を取得
    pub fn list_objects(&self, prefix: &'a str) -> Result<Vec<String>> {
        // バケット名が一致するもののファイル名を一覧取得する
        let names = self.accessor.get_filenames()?
            .into_iter()
            .filter(|filename|
                filename.starts_with(Self::get_hex(&self.bucket_name).as_str())
            );
        // 付属情報に変換する
        let attrs = names.into_iter()
            .map(|name| Attributes::get_path(&name))
            .filter_map(|path| {
                let reader = self.accessor.get_reader(&path);
                match reader.read() {
                    Ok(bytes) => Attributes::try_from(bytes).ok(),
                    _ => None
                }
            });
        // マッチする名前で検索する
        let names = attrs
            .map(|attribute| attribute.name())
            .filter(|name| name.starts_with(prefix))
            .collect::<Vec<_>>();
        Ok(names)
    }
}
