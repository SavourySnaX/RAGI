
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

//    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    if index != 47 {
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

    process_logic_slice(logic_slice);

    process_text_slice(text_slice);

    return Ok(String::from("POOP"));
}

fn process_text_slice(text_slice: &[u8]) {
    // unpack the text data first
    let mut iter = text_slice.iter();
    let num_messages = iter.next().unwrap();
    let lsb_pos = iter.next().unwrap();
    let msb_pos = iter.next().unwrap();
    let position:usize = *msb_pos as usize;
    let position = position<<8;
    let position = position + (*lsb_pos as usize);
    let _end_of_messages = position;
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
}

#[derive(PartialEq)]
enum LogicState {
    Action,
    Test,
    BracketStart,
}

#[derive(Copy,Clone,PartialEq)]
enum Params {
    Flag,
    Num,
    None,
}

fn process_logic_slice(logic_slice: &[u8]) {

    let mut iter = logic_slice.iter();

    let mut state=LogicState::Action;
    let mut params = [Params::None; 7];
    let mut param_idx = 0;
    let mut indent = false;
    let mut bracket_stack:Vec<u16>=Vec::new();

    while let Some(b) = iter.next()
    {
        if params[param_idx] != Params::None {
            match params[param_idx] {
                Params::Flag => { print!("flag:{}",*b); }
                Params::Num => { print!("{}",*b); }
                Params::None => panic!("Should not be reached"),
            }
            params[param_idx]=Params::None;
            param_idx+=1;
            if params[param_idx] == Params::None {
                param_idx=0;
                if state == LogicState::Test {
                    print!(")");
                } else {
                    println!(");");
                    indent=true;
                }

            } else {
                print!(",");
            }
        } else {
            match state {
                LogicState::Test => {
                    match b {
                        0xFF => { println!(")"); state=LogicState::BracketStart; indent=true;},
                        0x07 => { print!("isset("); params[0]=Params::Flag; }
                        _ => panic!("Unhandled Test Command Type {:02X}",*b),
                    }
                },
                LogicState::Action => {
                    match b {
                        0xFF => { print!("if ("); state=LogicState::Test; },
                        0x14 => { print!("load.logics("); params[0]=Params::Num; }
                        0x16 => { print!("call("); params[0]=Params::Num; }
                        0x00 => { println!("return;"); indent=true; }
                        _ => panic!("Unhandled Action Command Type {:02X}", *b),
                    }
                },
                LogicState::BracketStart => {   // TODO split into two to avoid needing logic to handle decrement by 2 for bracket_stack?
                    let lsb:u16 = (*b).into();
                    let msb:u16 = (*iter.next().unwrap()).into();
                    let pos:u16 = (msb<<8)+lsb+1;
                    bracket_stack.push(pos);
                    println!("{{"); indent=true;
                    state=LogicState::Action;
                },
                _ => panic!("TODO"),
            }
            if indent {print!("{:indent$}","",indent=bracket_stack.len()); indent=false;}
        }
        if !bracket_stack.is_empty() {
            let pos = bracket_stack.len()-1;
            let mut cnt = bracket_stack[pos];
            cnt-=1;
            if cnt==0 {
                bracket_stack.pop();
                println!("}}"); indent=true;
            } else {
                bracket_stack[pos]=cnt;
            }
        }
    }

}
/*
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
*/