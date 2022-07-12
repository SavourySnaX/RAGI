

pub struct Volume {
    pub data:Vec<u8>,
}

impl Volume {
    pub fn new(bytes: impl Iterator<Item = u8>) -> Result<Volume,&'static str> {
        return Ok(Volume {data:bytes.collect()});
    }
}