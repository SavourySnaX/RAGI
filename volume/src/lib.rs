use std::collections::HashMap;

use dir_resource::{ResourceDirectoryEntry, ResourceCompression};

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
    
    pub fn fetch_data_slice<'a>(&'a self,cache:&'a mut VolumeCache, entry: &ResourceDirectoryEntry) -> Result<(&'a [u8],ResourceCompression),&'static str> {

        return match entry.compression {
            ResourceCompression::None => self.fetch_data_slice_v2(entry),
            ResourceCompression::LZW | ResourceCompression::Picture => self.fetch_data_slice_v3(cache,entry),
        };
    }
    
    fn fetch_data_slice_v2<'a>(&'a self, entry: &ResourceDirectoryEntry) -> Result<(&'a [u8],ResourceCompression),&'static str> {

        if (entry.position+5) as usize > self.data.len() {
            return Ok((&[],ResourceCompression::None));
        }
        let slice = &self.data[entry.position as usize..];
        let slice = &slice[3..]; // Skip 0x1234 + Vol

        let length:usize = slice[0].into();
        let upper:usize = slice[1].into();
        let upper = upper<<8;
        let length = length+upper;
        Ok((&slice[2..length+2],ResourceCompression::None))
    }

    fn fetch_data_slice_v3<'a>(&'a self, cache:&'a mut VolumeCache,entry: &ResourceDirectoryEntry) -> Result<(&'a [u8],ResourceCompression),&'static str> {

        if (entry.position+7) as usize > self.data.len() {
            return Ok((&[],ResourceCompression::None));
        }
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

        match entry.compression {
            ResourceCompression::None => Err("Should not reach here for uncompressed entry"),
            ResourceCompression::Picture => Ok((&slice[4..compressed_length+4],ResourceCompression::Picture)),
            ResourceCompression::LZW => {
                if compressed_length == uncompressed_length {
                    return Ok((&slice[4..uncompressed_length+4], ResourceCompression::None));
                }
                let cache_entry:usize=entry.position as usize;
                let cache_entry = cache_entry + ((entry.volume as usize)<<32);
                if !cache.cache.contains_key(&cache_entry) {
                    let bytes = &slice[4..compressed_length+4];
                    let mut output:Vec<u8> = Vec::new();
                    output.reserve(uncompressed_length);
                    let decoded = agi_lzw_expand(bytes, &mut output);
                    if decoded.is_ok() {
                        let result_size=output.len();
                        if uncompressed_length==result_size {
                            cache.cache.insert(cache_entry, output);
                        } else {
                            return Err("Failed to decompress, final size != expected length");
                        }
                    } else {
                        return Err("Failed to decompress LZW encoded");
                    }
                }
                let v=&cache.cache[&cache_entry];
                Ok((v.as_slice(), ResourceCompression::LZW))
            },
        }
    }

}

const TABLE_SIZE:usize = 4096;

struct LzwState {
    bit_buffer:u8,
    bit_remain:u8,
    num_bits:u32,
    max_code:u32,
    next_code:u32,
    decoded_byte:[u8;TABLE_SIZE],
    code_follow:[u32;TABLE_SIZE],
}

impl LzwState {
    fn new(start_bits:u32,next_code:u32) -> LzwState {
        let mut state = LzwState {
            bit_buffer:0,
            bit_remain:0,
            num_bits:0,
            next_code,
            max_code:0,
            decoded_byte:[0u8;TABLE_SIZE],
            code_follow:[0u32;TABLE_SIZE],
        };
        state.set_bits(start_bits);
        state
    }

    fn set_bits(&mut self,size:u32) {
        if size>=12 {
            return;
        }
        self.num_bits=size;
        self.max_code=(1<<size)-2;
    }

    fn get_code(&mut self,input_slice:&[u8]) -> (u32,usize) {

        let mut input_pos=0;
        let mut code:u32=0;
        let mut shifter=0;
        while shifter!=self.num_bits {

            if self.bit_remain>0 {
                code|=((self.bit_buffer&1) as u32)<<shifter;
                self.bit_buffer>>=1u8;
                self.bit_remain-=1;
                shifter+=1;
            } else {
                // refill
                self.bit_buffer = input_slice[input_pos];
                self.bit_remain = 8;
                input_pos+=1;
            }
        }

        (code,input_pos)
    }

    fn decode(&self,output:&mut Vec<u8>,start:u32,include:bool,c:u8) -> u8 {
        let mut t:Vec<u8> = Vec::new();
        if include{
            t.push(c);
        }
        let mut code = start;
        while code>255 {
            t.push(self.decoded_byte[code as usize]);
            code=self.code_follow[code as usize];
        }

        t.push(code as u8);
        for n in t.iter().rev() {
            output.push(*n);
        }

        t[t.len()-1]
    }

    fn update_code(&mut self,code:u32,byte:u8) {
        if self.next_code > self.max_code {
            self.set_bits(self.num_bits+1);
        }

        self.code_follow[self.next_code as usize]=code;
        self.decoded_byte[self.next_code as usize]=byte;
        self.next_code+=1;
    }

    fn reset(&mut self,start_bits:u32,next_code:u32) {
        self.set_bits(start_bits);
        self.next_code=next_code;
    }

    fn is_input_remaining(&self,slice:&[u8]) -> bool {
        (!slice.is_empty()) || self.bit_remain!=0
    }
    
}

fn agi_lzw_expand(input_slice:&[u8],output:&mut Vec<u8>) -> Result<(),&'static str> {

    let mut state=LzwState::new(9,257);

    let (mut last_code,slice_pos) = state.get_code(input_slice);
    let mut input_slice=&input_slice[slice_pos..];
    let mut c=last_code as u8;
    while state.is_input_remaining(input_slice) {
        let (next_code,slice_pos) = state.get_code(input_slice);
        input_slice=&input_slice[slice_pos..];

        match next_code {
            256 => {
                state.reset(9,258);
                let (next_code,slice_pos) = state.get_code(input_slice);
                input_slice=&input_slice[slice_pos..];
                last_code=next_code;
                c=last_code as u8;
                output.push(c);
                continue;
            },
            257 => return Ok(()),
            _ => {
                if next_code <state.next_code {
                    c=state.decode(output,next_code,false,c);
                } else {
                    c=state.decode(output,last_code,true,c);
                }
            },
        };

        state.update_code(last_code,c);
        last_code=next_code;
    }

    Err("Ran out of input buffer, before hitting a 257 end of input")
}