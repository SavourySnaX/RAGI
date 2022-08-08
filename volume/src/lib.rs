use std::collections::HashMap;

use dir_resource::{ResourceDirectoryEntry, ResourceCompression};
use weezl::{BitOrder,decode::Decoder};

pub struct Volume {
    pub data:Vec<u8>,
}

pub struct VolumeCache {
    cache:HashMap<usize,Vec<u8>>,
}

impl VolumeCache {
    pub fn new() -> VolumeCache {
        VolumeCache { cache:HashMap::new() }
    }
}

impl Volume {
    pub fn new(bytes: impl Iterator<Item = u8>) -> Result<Volume,&'static str> {
        Ok(Volume {data:bytes.collect()})
    }
    
    pub fn fetch_data_slice<'a>(&'a self,cache:&'a mut VolumeCache, entry: &ResourceDirectoryEntry) -> Result<&'a [u8],&'static str> {

        return match entry.compression {
            ResourceCompression::None => self.fetch_data_slice_v2(entry),
            ResourceCompression::LZW | ResourceCompression::Picture => self.fetch_data_slice_v3(cache,entry),
        };
    }
    
    fn fetch_data_slice_v2<'a>(&'a self, entry: &ResourceDirectoryEntry) -> Result<&'a [u8],&'static str> {

        let slice = &self.data[entry.position as usize..];
        let slice = &slice[3..]; // Skip 0x1234 + Vol

        let length:usize = slice[0].into();
        let upper:usize = slice[1].into();
        let upper = upper<<8;
        let length = length+upper;
        Ok(&slice[2..length+2])
    }

    fn fetch_data_slice_v3<'a>(&'a self, cache:&'a mut VolumeCache,entry: &ResourceDirectoryEntry) -> Result<&'a [u8],&'static str> {

        let slice = &self.data[entry.position as usize..];
        let slice = &slice[3..]; // Skip 0x1234 + Vol (note we skip the upper bit in vol that indicates picture resource, assuming the compression kind to be enough)

        let length:usize = slice[0].into();
        let upper:usize = slice[1].into();
        let upper = upper<<8;
        let uncompressed_length = length+upper;
        let length:usize = slice[2].into();
        let upper:usize = slice[3].into();
        let upper = upper<<8;
        let compressed_length = length+upper;

        if compressed_length == uncompressed_length {
            return Ok(&slice[4..uncompressed_length+4]);
        }

        match entry.compression {
            ResourceCompression::None => Err("Should not reach here for uncompressed entry"),
            ResourceCompression::Picture => Ok(&slice[4..compressed_length+4]),
            ResourceCompression::LZW => {
                let cache_entry:usize=entry.position as usize;
                let cache_entry = cache_entry + (entry.volume as usize)<<32;
                if !cache.cache.contains_key(&cache_entry) {
                    let bytes = &slice[4..compressed_length+4];
                    let mut decoder = Decoder::new(BitOrder::Lsb,9);
                    let decoded = decoder.decode(bytes);
                    if let Ok(result) = decoded {
                        let result_size=result.len();
                        if compressed_length==result_size {
                            cache.cache.insert(cache_entry, result);
                        } else {
                            return Err("bljkasjha");
                        }
                    } else {
                        if let Err(error) = decoded {
                            println!("{}",error);
                        }
                        return Err("Failed to decompress LZW encoded");
                    }
                }
                let v=&cache.cache[&cache_entry];
                Ok(v.as_slice())
            },
        }
    }

}