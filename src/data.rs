use std::{convert::TryInto, fs, mem, path};
use crate::config::Config;

fn read_index_file(config: &Config, collection_name: &str) -> Vec<u8> {
    let index_file_path = path::Path::new(&config.state_location).join(&collection_name).join("index");
    fs::read(index_file_path).unwrap()
}

pub fn parse_index_raw_data(data: Vec<u8>, collection_name: &str) -> Vec<u128> {
    let entry_size = mem::size_of::<u128>();
    let data_size = data.len();
    if data_size % entry_size != 0 {
        panic!("Size of index data for collection \"{}\" must be a multiple of {} bytes, instead found {} bytes!", collection_name, entry_size, data_size);
    }
    data.chunks(entry_size).map(|chunk| u128::from_be_bytes(chunk.try_into().unwrap())).collect()
}

pub fn get_index_data(config: &Config, collection_name: &str) -> Vec<u128> {
    let raw_data = read_index_file(&config, &collection_name);
    let data = parse_index_raw_data(raw_data, &collection_name);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_data_parsing_works() {
        let raw_data: Vec<u8> = vec![0xf0, 0x0f, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff];

        assert_eq!(
            parse_index_raw_data(raw_data, "test"), [0xf00f0000ffff0000ffff000000ffffff]
        );
    }

    #[test]
    #[should_panic(expected = "Size of index data for collection \"test\" must be a multiple of 16 bytes, instead found 17 bytes!")]
    fn raw_data_parsing_panics_when_data_is_wrong_size() {
        let raw_data: Vec<u8> = vec![0xf0, 0x0f, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00];
        
        parse_index_raw_data(raw_data, "test");
    }
}
