use std::{collections::{HashMap}, convert::TryFrom};
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// ファイルに付属するデータを格納する
/// `children` が子の名前だけで、パスなのかアイテムなのかは持たない -> まずいかも
#[derive(Serialize, Deserialize, Debug)]
pub struct Attributes {
    bucket: String,
    name: String,
    meta: HashMap<String, String>,
}

impl Attributes {
    pub fn new(bucket: String, name: String) -> Attributes {
        Attributes {
            bucket,
            name,
            meta: HashMap::new(),
        }
    }

    /// パスを取得する
    pub fn get_path<'a>(object_path: &'a str) -> String {
        format!("{}.json", object_path)
    }

    pub fn add_meta(&mut self, key: String, value: String) {
        self.meta.insert(key, value);
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl TryFrom<Vec<u8>> for Attributes {
    type Error = serde_json::error::Error;
    /// バイト列から読み込む
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&bytes)
    }
}

impl TryFrom<Attributes> for Vec<u8> {
    type Error = serde_json::error::Error;
    fn try_from(value: Attributes) -> Result<Self, Self::Error> {
        serde_json::to_vec(&value)
    }
}

pub fn is_users_meta_key(key: String) -> bool {
    use regex::Regex;
    let re = Regex::new("^x-amn-meta-.+").unwrap();
    re.is_match(&key)
}
