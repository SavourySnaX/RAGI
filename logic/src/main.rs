
use std::fs;
use std::{path::Path};

use dir_resource::{ResourceDirectory, ResourceDirectoryEntry};
use volume::Volume;

struct Root<'a> {
    base_path:&'a Path,
}

impl<'a> Root<'_> {
    pub fn new(base_path:&'a str) -> Root {
        Root {base_path:Path::new(base_path)}
    }
}

fn main() {

    //let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/");
    let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/");

    let bytes = fs::read(root.base_path.join("LOGDIR").into_os_string()).unwrap_or_default();

    let dir = ResourceDirectory::new(bytes.into_iter()).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_logic_resource(&root,&entry, index);
        }
    }

}

fn dump_logic_resource(root:&Root,entry:&ResourceDirectoryEntry, index:usize) {

    let bytes = fs::read(root.base_path.join(format!("VOL.{}", entry.volume)).into_os_string()).unwrap_or_default();

    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data = fetch_data_slice(&volume, entry);

    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    if index == 999999 {
        return;
    }

    let logic = match process_logic(index, &volume, entry) {
        Ok(b) => b,
        Err(s) => panic!("Failed due to : {}", s),
    };

}

//todo upgrade to Result
fn fetch_data_slice<'a>(volume: &'a Volume, entry: &ResourceDirectoryEntry) -> &'a [u8] {

    let slice = &volume.data[entry.position as usize..];
    let slice = &slice[3..]; // Skip 0x1234 + Vol

    let length:usize = slice[0].into();
    let upper:usize = slice[1].into();
    let upper = upper<<8;
    let length = length+upper;
    &slice[2..length+2]
}

fn process_logic(index:usize, volume:&Volume, entry: &ResourceDirectoryEntry) -> Result<String, String> {

    let slice = fetch_data_slice(volume, entry);
    let mut slice_iter = slice.iter();

    let lsb_pos = slice_iter.next().unwrap();
    let msb_pos = slice_iter.next().unwrap();
    let position:usize = *msb_pos as usize;
    let position = position<<8;
    let position = position + (*lsb_pos as usize);
    let text_start = position;

    let logic_slice = &slice[2..text_start+2];
    let text_slice = &slice[text_start+2..];

    // unpack the text data first
    let mut iter = text_slice.iter();
    let num_messages = iter.next().unwrap();
    
    let lsb_pos = iter.next().unwrap();
    let msb_pos = iter.next().unwrap();
    let position:usize = *msb_pos as usize;
    let position = position<<8;
    let position = position + (*lsb_pos as usize);
    let end_of_messages = position;
    let decrypt_start_adjust:usize = 2;

    let mut messages:Vec<usize> = Vec::new();
    messages.reserve((*num_messages).into());
    for _m in 0..*num_messages {
        let lsb_pos = iter.next().unwrap();
        let msb_pos = iter.next().unwrap();
        let position:usize = *msb_pos as usize;
        let position = position<<8;
        let position = position + (*lsb_pos as usize);
        messages.push(position);
    }

    let decrypt = "Avis Durgan";
    let decrypt_start_adjust = decrypt_start_adjust + messages.len()*2;
    let message_block_slice = &text_slice[1..];

    for m in messages {
        let mut string = String::new();
        if m!=0 {
            let mut decrypt_iter = decrypt.bytes().cycle().skip(m-decrypt_start_adjust);
            let slice = &message_block_slice[m..];
            let mut iter = slice.iter();
            loop {
                let n = iter.next().unwrap();
                let decrypted = n ^ decrypt_iter.next().unwrap();
                if decrypted == 0 {
                    break;
                }
                string = string + &String::from(decrypted as char);
            }
        }

        println!("{}",string);
    }

    return Ok(String::from("POOP"));
}
