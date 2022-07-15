
use std::{fs};
use std::{path::Path};

use dir_resource::{ResourceDirectory, ResourceDirectoryEntry};
use objects::Objects;
use volume::Volume;
use words::Words;

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

    let bytes = fs::read(root.base_path.join("OBJECT").into_os_string()).unwrap_or_default();

    let items = Objects::new(&bytes).unwrap();
    
    let bytes = fs::read(root.base_path.join("WORDS.TOK").into_os_string()).unwrap_or_default();

    let words = Words::new(bytes.into_iter()).unwrap();

    let bytes = fs::read(root.base_path.join("LOGDIR").into_os_string()).unwrap_or_default();

    let dir = ResourceDirectory::new(bytes.into_iter()).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_logic_resource(&root,&entry,index,&items,&words);
        }
    }

}

fn dump_logic_resource(root:&Root,entry:&ResourceDirectoryEntry,index:usize,items:&Objects,words:&Words) {

    let bytes = fs::read(root.base_path.join(format!("VOL.{}", entry.volume)).into_os_string()).unwrap_or_default();

    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data = fetch_data_slice(&volume, entry);

    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    let logic = match process_logic(index, &volume, entry,&items,&words) {
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

fn process_logic(index:usize, volume:&Volume, entry: &ResourceDirectoryEntry, items:&Objects, words:&Words) -> Result<String, String> {

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

    let logic_messages = process_text_slice(text_slice);

    process_logic_slice(logic_slice,&logic_messages,&items,&words);

    return Ok(String::from("POOP"));
}

pub struct LogicMessages {
    pub strings:Vec<String>,
}

fn process_text_slice(text_slice: &[u8]) -> LogicMessages {
    // unpack the text data first
    let mut strings:Vec<String> = Vec::new();
    strings.push("".to_string());   // Push [0] "" string, since messages start counting from 1

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

        strings.push(string);
    }

    return LogicMessages { strings };
}

#[derive(PartialEq,Debug)]
enum LogicState {
    Action,
    Test,
    TestOr,
    BracketStart,
    BracketEnd,
    GotoStart,
    GotoEnd,
}

#[derive(Copy,Clone,PartialEq)]
enum Params {
    Flag,
    Num,
    Var,
    Object,
    Controller,
    Message,
    String,
    Said,
    WordLSB,
    WordMSB,
    Item,
    None,
}

fn print_words(words:&Words,word_num:u16) {
    print!("word:{}",word_num);
    if word_num == 1 {
        print!("<any>");
    } else if word_num == 9999 {
        print!("<rest of line>");
    } else {
        let match_words = words.fetch_all(word_num);
        print!(" (");
        for (index,word) in match_words.into_iter().enumerate() {
            if index !=0 {
                print!(" || ");
            }
            print!("\"{}\"",word);
        }
        print!(" )");
    }
}

fn process_logic_slice(logic_slice: &[u8],logic_messages:&LogicMessages,items:&Objects,words:&Words) {

    let mut iter = logic_slice.iter();

    let mut state=LogicState::Action;
    let mut params = [Params::None; 80];
    let mut param_idx = 0;
    let mut indent = false;
    let mut bracket_label_stack:Vec<(u16,u32)>=Vec::new();
    let mut bracket_label_indent=1;
    let mut bracket_lsb:u16 = 0;
    let mut expression_continue=false;
    let mut next_label=1;

// TODO - doesn't handle backwards gotos (label will be missing)

    print!(" ");
    while let Some(b) = iter.next()
    {
        if params[param_idx] != Params::None {
            match params[param_idx] {
                Params::Flag => { print!("flag:{}",*b); },
                Params::Num => { print!("{}",*b); },
                Params::Var => { print!("var:{}",*b); },
                Params::Object => { print!("obj:{}",*b); },
                Params::Item => { print!("item:{}\"{}\"",*b, items.objects[(*b)as usize].name); },
                Params::Controller => { print!("ctr:{}",*b); },
                Params::Message => { print!("msg:{}\"{}\"",*b,logic_messages.strings[(*b) as usize]); },
                Params::String => { print!("str:{}",*b); },
                Params::WordLSB => { bracket_lsb = (*b).into(); },
                Params::WordMSB => { let msb:u16 = (*b).into(); let word:u16=(msb<<8) + bracket_lsb; print_words(words,word); },
                Params::Said => { for w in 0..*b as usize { (params[1+w*2+0],params[1+w*2+1])=(Params::WordLSB,Params::WordMSB); }},
                Params::None => panic!("Should not be reached"),
            }
            let last = params[param_idx];
            params[param_idx]=Params::None;
            param_idx+=1;
            if params[param_idx] == Params::None {
                param_idx=0;
                match state {
                    LogicState::Test | LogicState::TestOr => {
                        print!(")");
                    },
                    LogicState::Action => {
                        println!(");");
                        indent=true;
                    },
                    _ => panic!("unexpected logic state {:?}",state),
                }
            } else {
                match last {
                    Params::WordLSB | Params::Said => {},
                    _ => print!(","),
                }
            }
        } else {
            match state {
                LogicState::TestOr => {
                    match b {
                        0xFD => { if expression_continue { print!(" || "); }  print!("!"); expression_continue=false; },
                        0xFC => { print!(")"); state = LogicState::Test },
                        0x12 => { if expression_continue { print!(" || "); } print!("right.posn("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Num,Params::Num,Params::Num,Params::Num); expression_continue=true; },
                        0x10 => { if expression_continue { print!(" || "); } print!("obj.in.box("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Num,Params::Num,Params::Num,Params::Num); expression_continue=true; },
                        0x0E => { if expression_continue { print!(" || "); } print!("said("); params[0]=Params::Said; expression_continue=true; },
                        0x0D => { if expression_continue { print!(" || "); } print!("have.key()"); expression_continue=true; },
                        0x0C => { if expression_continue { print!(" || "); } print!("controller("); params[0]=Params::Controller; expression_continue=true; },
                        0x0B => { if expression_continue { print!(" || "); } print!("posn("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Num,Params::Num,Params::Num,Params::Num); expression_continue=true; },
                        0x09 => { if expression_continue { print!(" || "); } print!("has("); params[0]=Params::Item; expression_continue=true; },
                        0x08 => { if expression_continue { print!(" || "); } print!("issetv("); params[0]=Params::Var; expression_continue=true; },
                        0x07 => { if expression_continue { print!(" || "); } print!("isset("); params[0]=Params::Flag; expression_continue=true; },
                        0x06 => { if expression_continue { print!(" || "); } print!("greaterv("); (params[0],params[1])=(Params::Var,Params::Var); expression_continue=true; },
                        0x05 => { if expression_continue { print!(" || "); } print!("greatern("); (params[0],params[1])=(Params::Var,Params::Num); expression_continue=true; },
                        0x04 => { if expression_continue { print!(" || "); } print!("lessv("); (params[0],params[1])=(Params::Var,Params::Var); expression_continue=true; },
                        0x03 => { if expression_continue { print!(" || "); } print!("lessn("); (params[0],params[1])=(Params::Var,Params::Num); expression_continue=true; },
                        0x02 => { if expression_continue { print!(" || "); } print!("equalv("); (params[0],params[1])=(Params::Var,Params::Var); expression_continue=true; },
                        0x01 => { if expression_continue { print!(" || "); } print!("equaln("); (params[0],params[1])=(Params::Var,Params::Num); expression_continue=true; },
                        _ => panic!("Unhandled Test Command Type {:02X}",*b),
                    }
                },
                LogicState::Test => {
                    match b {
                        0xFF => { println!(")"); state=LogicState::BracketStart; indent=true;},
                        0xFD => { if expression_continue { print!(" && "); } print!("!"); expression_continue=false; },
                        0xFC => { if expression_continue { print!(" && "); } print!("("); state=LogicState::TestOr; expression_continue=false; },
                        0x12 => { if expression_continue { print!(" && "); } print!("right.posn("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Num,Params::Num,Params::Num,Params::Num); expression_continue=true; },
                        0x10 => { if expression_continue { print!(" && "); } print!("obj.in.box("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Num,Params::Num,Params::Num,Params::Num); expression_continue=true; },
                        0x0E => { if expression_continue { print!(" && "); } print!("said("); params[0]=Params::Said; expression_continue=true; },
                        0x0D => { if expression_continue { print!(" && "); } print!("have.key()"); expression_continue=true; },
                        0x0C => { if expression_continue { print!(" && "); } print!("controller("); params[0]=Params::Controller; expression_continue=true; },
                        0x0B => { if expression_continue { print!(" && "); } print!("posn("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Num,Params::Num,Params::Num,Params::Num); expression_continue=true; },
                        0x09 => { if expression_continue { print!(" && "); } print!("has("); params[0]=Params::Item; expression_continue=true; },
                        0x08 => { if expression_continue { print!(" && "); } print!("issetv("); params[0]=Params::Var; expression_continue=true; },
                        0x07 => { if expression_continue { print!(" && "); } print!("isset("); params[0]=Params::Flag; expression_continue=true; },
                        0x06 => { if expression_continue { print!(" && "); } print!("greaterv("); (params[0],params[1])=(Params::Var,Params::Var); expression_continue=true; },
                        0x05 => { if expression_continue { print!(" && "); } print!("greatern("); (params[0],params[1])=(Params::Var,Params::Num); expression_continue=true; },
                        0x04 => { if expression_continue { print!(" && "); } print!("lessv("); (params[0],params[1])=(Params::Var,Params::Var); expression_continue=true; },
                        0x03 => { if expression_continue { print!(" && "); } print!("lessn("); (params[0],params[1])=(Params::Var,Params::Num); expression_continue=true; },
                        0x02 => { if expression_continue { print!(" && "); } print!("equalv("); (params[0],params[1])=(Params::Var,Params::Var); expression_continue=true; },
                        0x01 => { if expression_continue { print!(" && "); } print!("equaln("); (params[0],params[1])=(Params::Var,Params::Num); expression_continue=true; },
                        _ => panic!("Unhandled Test Command Type {:02X}",*b),
                    }
                },
                LogicState::Action => {
                    match b {
                        0xFF => { print!("if ("); state=LogicState::Test; expression_continue=false; },
                        0xFE => { print!("goto "); state=LogicState::GotoStart; }
                        0x97 => { print!("print.at("); (params[0],params[1],params[2])=(Params::Message,Params::Num,Params::Num); }, // Version 2.400 onwards has 1 extra numerical argument
                        0x96 => { print!("trace.info("); (params[0],params[1],params[2])=(Params::Num,Params::Num,Params::Num); },
                        0x94 => { print!("reposition.to.v("); (params[0],params[1],params[2])=(Params::Object,Params::Var,Params::Var); },
                        0x93 => { print!("reposition.to("); (params[0],params[1],params[2])=(Params::Object,Params::Num,Params::Num); },
                        0x8F => { print!("set.game.id("); params[0]=Params::Message;},
                        0x8D => { println!("version();"); indent=true; },
                        0x8C => { println!("toggle.monitor();"); indent=true; },
                        0x8B => { println!("init.joy();"); indent=true; },
                        0x8A => { println!("cancel.line();"); indent=true; },
                        0x89 => { println!("echo.line();"); indent=true; },
                        0x88 => { println!("pause();"); indent=true; },
                        0x87 => { println!("show.mem();"); indent=true; },
                        0x86 => { println!("quit();"); indent=true; },  // Version 2.272 onwards has 1 numerical argument
                        0x85 => { print!("obj.status.v("); params[0]=Params::Var;},
                        0x84 => { println!("player.control();"); indent=true; },
                        0x83 => { println!("program.control();"); indent=true; },
                        0x82 => { print!("random("); (params[0],params[1],params[2])=(Params::Num,Params::Num,Params::Var); },
                        0x81 => { print!("show.obj("); params[0]=Params::Num;},
                        0x80 => { println!("restart.game();"); indent=true; },
                        0x7E => { println!("restore.game();"); indent=true; },
                        0x7D => { println!("save.game();"); indent=true; },
                        0x7C => { println!("status();"); indent=true; },
                        0x7B => { print!("add.to.pic.v("); (params[0],params[1],params[2],params[3],params[4],params[5],params[6])=(Params::Var,Params::Var,Params::Var,Params::Var,Params::Var,Params::Var,Params::Var); },
                        0x7A => { print!("add.to.pic("); (params[0],params[1],params[2],params[3],params[4],params[5],params[6])=(Params::Num,Params::Num,Params::Num,Params::Num,Params::Num,Params::Num,Params::Num); },
                        0x79 => { print!("set.key("); (params[0],params[1],params[2])=(Params::Num,Params::Num,Params::Controller); },
                        0x78 => { println!("accept.input();"); indent=true; },
                        0x77 => { println!("prevent.input();"); indent=true; },
                        0x76 => { print!("get.num("); (params[0],params[1])=(Params::Message,Params::Var); },
                        0x75 => { print!("parse("); params[0]=Params::String;},
                        0x73 => { print!("get.string("); (params[0],params[1],params[2],params[3],params[4])=(Params::String,Params::Message,Params::Num,Params::Num,Params::Num); },
                        0x72 => { print!("set.string("); (params[0],params[1])=(Params::String,Params::Message); },
                        0x71 => { println!("status.line.off();"); indent=true; },
                        0x70 => { println!("status.line.on();"); indent=true; },
                        0x6F => { print!("configure.screen("); (params[0],params[1],params[2])=(Params::Num,Params::Num,Params::Num); },
                        0x6E => { print!("shake.screen("); params[0]=Params::Num;},
                        0x6D => { print!("set.text.attribute("); (params[0],params[1])=(Params::Num,Params::Num); },
                        0x6C => { print!("set.cursor.char("); params[0]=Params::Message;},
                        0x6B => { println!("graphics();"); indent=true; },
                        0x6A => { println!("text.screen();"); indent=true; },
                        0x69 => { print!("clear.lines("); (params[0],params[1],params[2])=(Params::Num,Params::Num,Params::Num); },
                        0x68 => { print!("display.v("); (params[0],params[1],params[2])=(Params::Var,Params::Var,Params::Var); },
                        0x67 => { print!("display("); (params[0],params[1],params[2])=(Params::Num,Params::Num,Params::Message); },
                        0x66 => { print!("print.v("); params[0]=Params::Var;},
                        0x65 => { print!("print("); params[0]=Params::Message;},
                        0x64 => { println!("stop.sound();"); indent=true; },
                        0x63 => { print!("sound("); (params[0],params[1])=(Params::Num,Params::Flag); },
                        0x62 => { print!("load.sound("); params[0]=Params::Num;},
                        0x5E => { print!("drop("); params[0]=Params::Item;},
                        0x5D => { print!("get.v("); params[0]=Params::Var;},
                        0x5C => { print!("get("); params[0]=Params::Item;},
                        0x5B => { println!("unblock();"); indent=true; },
                        0x5A => { print!("block("); (params[0],params[1],params[2],params[3])=(Params::Num,Params::Num,Params::Num,Params::Num); },
                        0x59 => { print!("observe.blocks("); params[0]=Params::Object;},
                        0x58 => { print!("ignore.blocks("); params[0]=Params::Object;},
                        0x56 => { print!("set.dir("); (params[0],params[1])=(Params::Object,Params::Var);},
                        0x55 => { print!("normal.motion("); params[0]=Params::Object;},
                        0x54 => { print!("wander("); params[0]=Params::Object;},
                        0x53 => { print!("follow.ego("); (params[0],params[1],params[2])=(Params::Object,Params::Num,Params::Flag); },
                        0x52 => { print!("move.obj.v("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Var,Params::Var,Params::Num,Params::Flag); },
                        0x51 => { print!("move.obj("); (params[0],params[1],params[2],params[3],params[4])=(Params::Object,Params::Num,Params::Num,Params::Num,Params::Flag); },
                        0x50 => { print!("step.time("); (params[0],params[1])=(Params::Object,Params::Var);},
                        0x4F => { print!("step.size("); (params[0],params[1])=(Params::Object,Params::Var);},
                        0x4E => { print!("start.motion("); params[0]=Params::Object;},
                        0x4D => { print!("stop.motion("); params[0]=Params::Object;},
                        0x4C => { print!("cycle.time("); (params[0],params[1])=(Params::Object,Params::Var);},
                        0x4B => { print!("reverse.loop("); (params[0],params[1])=(Params::Object,Params::Flag);},
                        0x49 => { print!("end.of.loop("); (params[0],params[1])=(Params::Object,Params::Flag);},
                        0x47 => { print!("start.cycling("); params[0]=Params::Object;},
                        0x46 => { print!("stop.cycling("); params[0]=Params::Object;},
                        0x45 => { print!("distance("); (params[0],params[1],params[2])=(Params::Object,Params::Object,Params::Var); },
                        0x44 => { print!("observe.objs("); params[0]=Params::Object;},
                        0x43 => { print!("ignore.objs("); params[0]=Params::Object;},
                        0x41 => { print!("object.on.land("); params[0]=Params::Object;},
                        0x3F => { print!("set.horizon("); params[0]=Params::Num;},
                        0x3E => { print!("observe.horizon("); params[0]=Params::Object;},
                        0x3D => { print!("ignore.horizon("); params[0]=Params::Object;},
                        0x3C => { print!("force.update("); params[0]=Params::Object;},
                        0x3B => { print!("start.update("); params[0]=Params::Object;},
                        0x3A => { print!("stop.update("); params[0]=Params::Object;},
                        0x39 => { print!("get.priority("); (params[0],params[1])=(Params::Object,Params::Var);},
                        0x38 => { print!("release.priority("); params[0]=Params::Object;},
                        0x36 => { print!("set.priority("); (params[0],params[1])=(Params::Object,Params::Num);},
                        //0x35 => { print!("number.of.loops("); (params[0],params[1])=(Params::Object,Params::Var);},
                        0x34 => { print!("current.view("); (params[0],params[1])=(Params::Object,Params::Var); },
                        0x33 => { print!("current.loop("); (params[0],params[1])=(Params::Object,Params::Var); },
                        0x32 => { print!("current.cel("); (params[0],params[1])=(Params::Object,Params::Var); },
                        0x31 => { print!("last.cel("); (params[0],params[1])=(Params::Object,Params::Var); },
                        0x30 => { print!("set.cel.v("); (params[0],params[1])=(Params::Object,Params::Var); },
                        0x2F => { print!("set.cel("); (params[0],params[1])=(Params::Object,Params::Num); },
                        0x2E => { print!("release.loop("); params[0]=Params::Object; },
                        0x2D => { print!("fix.loop("); params[0]=Params::Object; },
                        0x2C => { print!("set.loop.v("); (params[0],params[1])=(Params::Object,Params::Var); },
                        0x2B => { print!("set.loop("); (params[0],params[1])=(Params::Object,Params::Num); },
                        0x2A => { print!("set.view.v("); (params[0],params[1])=(Params::Object,Params::Var); },
                        0x29 => { print!("set.view("); (params[0],params[1])=(Params::Object,Params::Num); },
                        0x28 => { print!("reposition("); (params[0],params[1],params[2])=(Params::Object,Params::Var,Params::Var); },
                        0x27 => { print!("get.posn("); (params[0],params[1],params[2])=(Params::Object,Params::Var,Params::Var); },
                        0x26 => { print!("position("); (params[0],params[1],params[2])=(Params::Object,Params::Var,Params::Var); },
                        0x25 => { print!("position("); (params[0],params[1],params[2])=(Params::Object,Params::Num,Params::Num); },
                        0x24 => { print!("erase("); params[0]=Params::Object; },
                        0x23 => { print!("draw("); params[0]=Params::Object; },
                        0x22 => { println!("unanimate.all();"); indent=true; },
                        0x21 => { print!("animate.obj("); params[0]=Params::Object; },
                        0x20 => { print!("discard.view("); params[0]=Params::Num; },
                        0x1F => { print!("load.view.v("); params[0]=Params::Var; },
                        0x1E => { print!("load.view("); params[0]=Params::Num; },
                        0x1D => { println!("show.pri.screen();"); indent=true; },
                        0x1B => { print!("discard.pic("); params[0]=Params::Var; },
                        0x1A => { println!("show.pic();"); indent=true; },
                        0x19 => { print!("draw.pic("); params[0]=Params::Var; },
                        0x18 => { print!("load.pic("); params[0]=Params::Var; },
                        0x17 => { print!("call.v("); params[0]=Params::Var; },
                        0x16 => { print!("call("); params[0]=Params::Num; },
                        0x14 => { print!("load.logics("); params[0]=Params::Num; },
                        0x13 => { print!("new.room.v("); params[0]=Params::Var; },
                        0x12 => { print!("new.room("); params[0]=Params::Num; },
                        0x10 => { print!("reset.v("); params[0]=Params::Var; },
                        0x0F => { print!("set.v("); params[0]=Params::Var; },
                        0x0E => { print!("toggle("); params[0]=Params::Flag; },
                        0x0D => { print!("reset("); params[0]=Params::Flag; },
                        0x0C => { print!("set("); params[0]=Params::Flag; },
                        0x0B => { print!("lindirectn("); (params[0],params[1])=(Params::Var,Params::Num); },
                        0x0A => { print!("rindirect("); (params[0],params[1])=(Params::Var,Params::Var); },
                        0x09 => { print!("lindirectv("); (params[0],params[1])=(Params::Var,Params::Var); },
                        0x08 => { print!("subv("); (params[0],params[1])=(Params::Var,Params::Var); },
                        0x07 => { print!("subn("); (params[0],params[1])=(Params::Var,Params::Num); },
                        0x06 => { print!("addv("); (params[0],params[1])=(Params::Var,Params::Var); },
                        0x05 => { print!("addn("); (params[0],params[1])=(Params::Var,Params::Num); },
                        0x04 => { print!("assignv("); (params[0],params[1])=(Params::Var,Params::Var); },
                        0x03 => { print!("assignn("); (params[0],params[1])=(Params::Var,Params::Num); },
                        0x02 => { print!("decrement("); params[0]=Params::Var; },
                        0x01 => { print!("increment("); params[0]=Params::Var; },
                        0x00 => { println!("return;"); indent=true; },
                        _ => panic!("Unhandled Action Command Type {:02X}", *b),
                    }
                },
                LogicState::BracketStart => {
                    bracket_lsb = (*b).into();
                    state=LogicState::BracketEnd;
                },
                LogicState::BracketEnd => {
                    let msb:u16 = (*b).into();
                    let pos:u16 = (msb<<8)+bracket_lsb;
                    bracket_label_stack.push((pos+1,0));  // +1 because will be decremented immediately
                    bracket_label_indent+=1;
                    println!("{{"); indent=true;
                    state=LogicState::Action;
                },
                LogicState::GotoStart => {
                    bracket_lsb = (*b).into();
                    state=LogicState::GotoEnd;
                },
                LogicState::GotoEnd => {
                    let msb:u16 = (*b).into();
                    let pos:u16 = (msb<<8)+bracket_lsb;
                    bracket_label_stack.push((pos+1,next_label));  // +1 because will be decremented immediately
                    let label = format!("LABEL_{:04}",next_label);
                    next_label+=1;
                    println!("{};",label); indent=true;
                    state=LogicState::Action;
                },
            }
        }
        if !bracket_label_stack.is_empty() {
            for s in (0..bracket_label_stack.len()).rev() {
                let (mut cnt,label) = bracket_label_stack[s];
                cnt-=1;
                if cnt==0 {
                    bracket_label_stack.remove(s);
                    if label!=0 {
                        let label = format!("LABEL_{:04}",label);
                        println!("{}:",label); indent=true;
                    } else {
                        bracket_label_indent-=1;
                        println!("{:indent$}}}","",indent=bracket_label_indent); indent=true;
                    }
                } else {
                    bracket_label_stack[s]=(cnt,label);
                }
            }
        }
        if indent {print!("{:indent$}","",indent=bracket_label_indent); indent=false;}
    }
}