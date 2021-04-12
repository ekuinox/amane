use std::collections::{HashMap, HashSet};
use std::fs::File;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// ファイルに付属するデータを格納する
/// `children` が子の名前だけで、パスなのかアイテムなのかは持たない -> まずいかも
#[derive(Serialize, Deserialize, Debug)]
pub struct Attributes {
    bucket: String,
    name: String,
    children: HashSet<String>,
    meta: HashMap<String, String>,
}

impl Attributes {
    pub fn new(bucket: String, name: String) -> Attributes {
        Attributes {
            bucket,
            name,
            children: HashSet::new(),
            meta: HashMap::new(),
        }
    }

    /// パスを取得する
    fn get_path(directory: String, bucket: String, name: String) -> String {
        format!("{}.json", super::get_path(&directory, &bucket, &name))
    }

    /// save meta
    pub fn save(&self, directory: String) -> Result<()> {
        use std::io::Write;
        let path = Self::get_path(directory, self.bucket.clone(), self.name.clone());
        let mut file = File::create(path)?;
        let deserialized = serde_json::to_string(self)?;
        let _ = file.write_all(&deserialized.as_bytes())?;
        Ok(())
    }

    /// meta from file
    /// これファイルをロックしてくれないので、並列でやれると詰むなー...
    pub fn from_file(directory: String, bucket: String, name: String) -> Result<Self> {
        let path = Self::get_path(directory, bucket, name.clone());
        Self::from_filepath(path)
    }

    pub fn from_filepath(path: String) -> Result<Self> {
        use std::io::BufReader;
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let attr = serde_json::from_reader(reader)?;
        Ok(attr)
    }

    /// get or new from file
    pub fn get_or_create(directory: String, bucket: String, name: String) -> Result<Self> {
        match Self::from_file(directory.clone(), bucket.clone(), name.clone()) {
            Ok(a) => {
                return Ok(a);
            },
            Err(_) => {
                let attr = Self::new(bucket.clone(), name.clone());
                match attr.save(directory) {
                    Ok(_) => Ok(attr),
                    Err(e) => Err(e),
                }
            }
        }
    }

    /// add child
    pub fn add_child(&mut self, child: String) {
        self.children.insert(child);
    }

    pub fn add_meta(&mut self, key: String, value: String) {
        self.meta.insert(key, value);
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub fn is_users_meta_key(key: String) -> bool {
    use regex::Regex;
    let re = Regex::new("^x-amn-meta-.+").unwrap();
    re.is_match(&key)
}
