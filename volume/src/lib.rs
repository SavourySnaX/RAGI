use dir_resource::ResourceDirectoryEntry;


pub struct Volume {
    pub data:Vec<u8>,
}

impl Volume {
    pub fn new(bytes: impl Iterator<Item = u8>) -> Result<Volume,&'static str> {
        return Ok(Volume {data:bytes.collect()});
    }
    
    pub fn fetch_data_slice<'a>(&'a self, entry: &ResourceDirectoryEntry) -> Result<&'a [u8],&'static str> {

        let slice = &self.data[entry.position as usize..];
        let slice = &slice[3..]; // Skip 0x1234 + Vol

        let length:usize = slice[0].into();
        let upper:usize = slice[1].into();
        let upper = upper<<8;
        let length = length+upper;
        Ok(&slice[2..length+2])
    }

}