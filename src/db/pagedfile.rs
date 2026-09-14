use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use crate::db::{Error, PAGE_SIZE};

#[derive(Debug)]
pub struct PagedFile {
    pub path: String,
}

impl PagedFile {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn create_if_not_exists(&self) -> Result<(), Error> {
        match File::create(&self.path) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::IO(e.to_string())),
        }
    }

    pub fn read_page(&self, page_no: usize) -> Result<[u8; PAGE_SIZE], Error> {
        let mut f = File::open(&self.path).map_err(|e| Error::IO(e.to_string()))?;
        f.seek(SeekFrom::Start((PAGE_SIZE * page_no) as u64))
            .map_err(|e| Error::IO(e.to_string()))?;
        let mut buf = [0u8; PAGE_SIZE];
        f.read_exact(&mut buf)
            .map_err(|e| Error::IO(e.to_string()))?;
        Ok(buf)
    }

    pub fn write_page(&self, page_no: usize, contents: &[u8]) -> Result<(), Error> {
        if contents.len() != PAGE_SIZE {
            return Err(Error::InvalidPageSize(contents.len()));
        }
        let mut f = File::open(&self.path).map_err(|e| Error::IO(e.to_string()))?;
        f.seek(SeekFrom::Start((PAGE_SIZE * page_no) as u64))
            .map_err(|e| Error::IO(e.to_string()))?;
        f.write_all(contents)
            .map_err(|e| Error::IO(e.to_string()))?;
        Ok(())
    }
}
