use anyhow::Result;
use std::fs::File;

#[derive(Clone, Debug)]
pub struct Accessor<'a> {
    directory: &'a str,
}

impl <'a, 'b> Accessor<'a> {
    pub fn new(directory: &'a str) -> Accessor {
        Accessor { directory }
    }

    fn get_path(&self, path: &'b str) -> String {
        format!("{}/{}", self.directory, path)
    }

    pub fn get_reader(&self, path: &'b str) -> Reader<'b> {
        Reader::new(path)
    }

    pub fn get_writer(&self, path: &'b str) -> Writer<'b> {
        Writer::new(path)
    }
}

pub struct Writer<'a> {
    path: &'a str,
}

impl <'a> Writer<'a> {
    pub(crate) fn new(path: &'a str) -> Writer<'a> {
        Writer { path }
    }

    /// バイト列を書き込む
    pub async fn write(&self, bytes: Vec<u8>) -> Result<()> {
        use std::io::Write;
        let mut file = File::open(self.path)?;
        let _ = file.write(&bytes)?;
        Ok(())
    }
}

pub struct Reader<'a> {
    path: &'a str,
}

impl <'a> Reader<'a> {
    pub(crate) fn new(path: &'a str) -> Reader<'a> {
        Reader { path }
    }

    /// バイト列を読み込む
    pub async fn read(&self) -> Result<Vec<u8>> {
        use std::io::Read;
        let mut file = File::open(self.path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}
