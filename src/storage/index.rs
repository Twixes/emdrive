use crate::config::Config;
use std::{
    convert::TryInto,
    fs,
    io::{self, Write},
    mem, path, process,
};
use tracing::*;

// Massively work-in-progress!

pub struct Index<'a> {
    collection_name: String,
    data: Vec<u128>,
    config: &'a Config,
}

impl<'a> Index<'a> {
    pub fn new(collection_name: &str, config: &'a Config) -> Self {
        let mut index = Index {
            collection_name: collection_name.to_string(),
            data: vec![],
            config,
        };
        index.sync_from_disk();
        index
    }

    pub fn get_data(&self) -> Vec<u128> {
        self.data.clone()
    }

    fn get_file_path(&self) -> path::PathBuf {
        path::Path::new(&self.config.data_directory)
            .join("data")
            .join(&self.collection_name)
            .join("index")
    }

    fn read_file(&self) -> io::Result<Vec<u8>> {
        let index_file_path = self.get_file_path();
        fs::read(index_file_path)
    }

    fn create_empty_file(&self) -> io::Result<fs::File> {
        let index_file_path = self.get_file_path();
        let index_dir_path = index_file_path.parent().unwrap();
        std::fs::create_dir_all(index_dir_path).unwrap_or_else(|_| panic!("Cannot create index file for collection {}, because its directory {} is improper! Consider changing setting data_directory (currently {})", &self.collection_name, index_dir_path.to_str().unwrap(), self.config.data_directory));
        fs::File::create(index_file_path)
    }

    fn parse_index_raw_data(&self, raw_data: Vec<u8>) -> Vec<u128> {
        let entry_size = mem::size_of::<u128>();
        let raw_data_size = raw_data.len();
        if raw_data_size % entry_size != 0 {
            panic!("Size of index data for collection `{}` must be a multiple of {} bytes, instead found {} bytes!", &self.collection_name, entry_size, raw_data_size);
        }
        raw_data
            .chunks(entry_size)
            .map(|chunk| u128::from_be_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    pub fn add(&mut self, value: u128) {
        self.data.push(value);
        self.sync_to_disk();
    }

    pub fn sync_from_disk(&mut self) -> Vec<u128> {
        let raw_data = self.read_file();
        let data = match raw_data {
            Ok(raw_data) => {
                let result = self.parse_index_raw_data(raw_data);
                debug!("Result: {:?}", result);
                result
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                self.create_empty_file().unwrap();
                vec![]
            }
            Err(err) => {
                debug!("Error while reading index data: {}", err);
                process::exit(1);
            }
        };
        debug!("Found {} index data: {:?}", &self.collection_name, &data);
        self.data = data;
        self.data.clone()
    }

    pub fn sync_to_disk(&mut self) {
        let mut file = self.create_empty_file().unwrap();
        let buffer: Vec<u8> = self
            .data
            .iter()
            .flat_map(|x| {
                let bytes: Vec<u8> = x.to_be_bytes().iter().copied().collect();
                bytes
            })
            .collect();
        file.write_all(buffer.as_slice()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn raw_data_parsing_works() {
        let dummy_index = Index {
            collection_name: "test".to_string(),
            data: vec![],
            config: &Config::default(),
        };

        let raw_data: Vec<u8> = vec![
            0xf0, 0x0f, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0xff,
            0xff, 0xff,
        ];

        assert_eq!(
            dummy_index.parse_index_raw_data(raw_data),
            [0xf00f0000ffff0000ffff000000ffffff]
        );
    }

    #[test]
    #[should_panic(
        expected = "Size of index data for collection `test` must be a multiple of 16 bytes, instead found 17 bytes!"
    )]
    fn raw_data_parsing_panics_when_data_is_wrong_size() {
        let dummy_index = Index {
            collection_name: "test".to_string(),
            data: vec![],
            config: &Config::default(),
        };

        let raw_data: Vec<u8> = vec![
            0xf0, 0x0f, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0xff,
            0xff, 0xff, 0x00,
        ];

        dummy_index.parse_index_raw_data(raw_data);
    }
}
