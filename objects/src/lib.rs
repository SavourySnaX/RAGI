
pub struct Object
{
    pub name:String,
    pub start_room:u8,
}

pub struct Objects
{
    pub max_objects:u8,
    pub objects:Vec<Object>,
}

impl Objects {

    pub fn blank() -> Objects {
        Objects { max_objects: 0x2A, objects:Vec::new()}
    }

    pub fn new(bytes: &[u8]) -> Result<Objects,&'static str> {

        let mut objects:Vec<Object> = Vec::new();

        let mut iter=bytes.iter();
        let slice = &bytes[3..];

        if let Some(b) = iter.next() {
            let lsb:usize = (*b).into();
            if let Some(b) = iter.next() {
                let msb:usize = (*b).into();
                let pos:usize = (msb<<8)+lsb;

                if pos>bytes.len() {
                    return Err("corrupted objects file");
                }

                let bytes_slice=&iter.as_slice()[..=pos];
                let mut iter = bytes_slice.iter();
                if let Some(b) = iter.next() {
                    let max_objects=*b;
                    while let Some(b) = iter.next() {
                        let lsb:usize = (*b).into();
                        if let Some(b) = iter.next() {
                            let msb:usize = (*b).into();
                            let pos:usize = (msb<<8)+lsb;
                            if let Some(b) = iter.next() {
                                let start_room = *b;
                                let name_slice = &slice[pos..];
                                let mut iter = name_slice.iter();
                                let mut name=String::new();
                                loop  {
                                    if let Some(b) = iter.next() {
                                        if *b==0 {
                                            break;
                                        }
                                        name = name + &String::from((*b)as char);
                                    }
                                    else {
                                        return Err("Failed to parse name");
                                    }
                                }
                                objects.push(Object {start_room, name});
                            } else {
                                return Err("Failed to read start room");
                            }
                        } else {
                            return Err("Expected MSB byte for name");
                        }
                    }
                    return Ok(Objects { max_objects, objects });
                }
                Err("Expected max objects byte")
            } else {
                Err("Expected offset to names")
            }
        } else {
            Err("Expected offset to names")
        }
    }
}
