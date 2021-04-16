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

    pub fn get_reader(&self, path: &'b str) -> Reader {
        Reader::new(self.get_path(path))
    }

    pub fn get_writer(&self, path: &'b str) -> Writer {
        Writer::new(self.get_path(path))
    }

    /// すでにファイルが存在しているか
    #[allow(dead_code)]
    pub fn is_exists(&self, path: &'b str) -> Result<bool> {
        match std::fs::File::open(self.get_path(path)) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn get_filenames(&self) -> Result<Vec<String>> {
        let filenames = std::fs::read_dir(self.directory)?
            .flatten()
            .map(|entry| entry.file_name().to_str().map(|name| name.to_string()))
            .flatten()
            .collect::<Vec<_>>();
        Ok(filenames)
    }
}

pub struct Writer {
    path: String,
}

impl Writer {
    pub(crate) fn new(path: String) -> Writer {
        Writer { path }
    }

    /// バイト列を書き込む
    pub fn write(&self, bytes: Vec<u8>) -> Result<()> {
        use std::io::Write;
        let mut file = File::create(&self.path)?;
        let _ = file.write_all(&bytes)?;
        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        let _ = std::fs::remove_file(&self.path)?;
        Ok(())
    }
}

pub struct Reader {
    path: String,
}

impl Reader {
    pub(crate) fn new(path: String) -> Reader {
        Reader { path }
    }

    /// バイト列を読み込む
    pub fn read(&self) -> Result<Vec<u8>> {
        use std::io::Read;
        let mut file = File::open(&self.path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}
