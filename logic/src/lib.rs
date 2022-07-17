use std::{collections::HashMap, hash::Hash, ops};

use dir_resource::ResourceDirectoryEntry;
use objects::Objects;
use volume::Volume;
use words::Words;

use std::convert::AsRef;
use strum_macros::AsRefStr;

pub struct LogicResource {
    logic_sequence:LogicSequence,
    logic_messages:LogicMessages,
}

pub struct LogicMessages {
    pub strings:Vec<String>,
}

pub struct LogicSequence {
    operations:Vec<ActionOperation>,
    labels:HashMap<TypeGoto,(bool,usize)>,
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

#[duplicate_item(name; [TypeFlag]; [TypeNum]; [TypeVar]; [TypeObject]; [TypeController]; [TypeMessage]; [TypeString]; [TypeItem])]
#[derive(Clone,Copy)]
#[allow(dead_code)]
pub struct name {
    value:u8,
}


#[derive(Clone,Copy)]
pub struct TypeWord {
    value:u16,
}

#[derive(Clone,Copy,Eq,Hash,PartialEq)]
pub struct TypeGoto {
    value:i16,
}

use duplicate::duplicate_item;

#[duplicate_item(name; [TypeFlag]; [TypeNum]; [TypeVar]; [TypeObject]; [TypeController]; [TypeMessage]; [TypeString]; [TypeItem])]
impl From<u8> for name {
    fn from(value: u8) -> Self {
        name {value}
    }
}

impl From<u16> for TypeWord {
    fn from(value: u16) -> Self {
        TypeWord {value}
    }
}

impl From<i16> for TypeGoto {
    fn from(value: i16) -> Self {
        TypeGoto {value}
    }
}

impl ops::Add<TypeGoto> for TypeGoto {
    type Output = TypeGoto;
    fn add(self, rhs:TypeGoto) -> Self::Output {
        let a:i16 = self.value.into();
        let b:i16 = rhs.value.into();
        return (a+b).into();
    }
}

#[derive(Copy,Clone,PartialEq)]
enum Operands {
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

#[derive(AsRefStr)]
pub enum LogicOperation {
    EqualN((TypeVar,TypeNum)),
    EqualV((TypeVar,TypeVar)),
    LessN((TypeVar,TypeNum)),
    LessV((TypeVar,TypeVar)),
    GreaterN((TypeVar,TypeNum)),
    GreaterV((TypeVar,TypeVar)),
    IsSet((TypeFlag,)),
    IsSetV((TypeVar,)),
    Has((TypeItem,)),
    PosN((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
    Controller((TypeController,)),
    HaveKey(()),
    Said((Vec<TypeWord>,)),
    ObjInBox((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
    RightPosN((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
}

#[derive(AsRefStr)]
pub enum LogicChange {
    #[strum(serialize = "")]
    Normal((LogicOperation,)),
    Not((LogicOperation,)),
    Or((Vec<LogicChange>,)),
}

#[derive(AsRefStr)]
pub enum ActionOperation {
    Return(()),
    Increment((TypeVar,)),
    Decrement((TypeVar,)),
    AssignN((TypeVar,TypeNum)),
    AssignV((TypeVar,TypeVar)),
    AddN((TypeVar,TypeNum)),
    AddV((TypeVar,TypeVar)),
    SubN((TypeVar,TypeNum)),
    SubV((TypeVar,TypeVar)),
    LIndirectV((TypeVar,TypeVar)),
    RIndirect((TypeVar,TypeVar)),
    LIndirectN((TypeVar,TypeNum)),
    Set((TypeFlag,)),
    Reset((TypeFlag,)),
    Toggle((TypeFlag,)),
    SetV((TypeVar,)),
    ResetV((TypeVar,)),
    NewRoom((TypeNum,)),
    NewRoomV((TypeVar,)),
    LoadLogic((TypeNum,)),
    Call((TypeNum,)),
    CallV((TypeVar,)),
    LoadPic((TypeVar,)),
    DrawPic((TypeVar,)),
    ShowPic(()),
    DiscardPic((TypeVar,)),
    ShowPriScreen(()),
    LoadView((TypeNum,)),
    LoadViewV((TypeVar,)),
    DiscardView((TypeNum,)),
    AnimateObj((TypeObject,)),
    UnanimateAll(()),
    Draw((TypeObject,)),
    Erase((TypeObject,)),
    Position((TypeObject,TypeNum,TypeNum)),
    PositionV((TypeObject,TypeVar,TypeVar)),
    GetPosN((TypeObject,TypeVar,TypeVar)),
    Reposition((TypeObject,TypeVar,TypeVar)),
    SetView((TypeObject,TypeNum)),
    SetViewV((TypeObject,TypeVar)),
    SetLoop((TypeObject,TypeNum)),
    SetLoopV((TypeObject,TypeVar)),
    FixLoop((TypeObject,)),
    ReleaseLoop((TypeObject,)),
    SetCel((TypeObject,TypeNum)),
    SetCelV((TypeObject,TypeVar)),
    LastCel((TypeObject,TypeVar)),
    CurrentCel((TypeObject,TypeVar)),
    CurrentLoop((TypeObject,TypeVar)),
    CurrentView((TypeObject,TypeVar)),
    SetPriority((TypeObject,TypeNum)),
    SetPriorityV((TypeObject,TypeVar)),
    ReleasePriority((TypeObject,)),
    GetPriority((TypeObject,TypeVar)),
    StopUpdate((TypeObject,)),
    StartUpdate((TypeObject,)),
    ForceUpdate((TypeObject,)),
    IgnoreHorizon((TypeObject,)),
    ObserveHorizon((TypeObject,)),
    SetHorizon((TypeNum,)),
    ObjectOnWater((TypeObject,)),
    ObjectOnLand((TypeObject,)),
    IgnoreObjs((TypeObject,)),
    ObserveObjs((TypeObject,)),
    Distance((TypeObject,TypeObject,TypeVar)),
    StopCycling((TypeObject,)),
    StartCycling((TypeObject,)),
    EndOfLoop((TypeObject,TypeFlag)),
    ReverseLoop((TypeObject,TypeFlag)),
    CycleTime((TypeObject,TypeVar)),
    StopMotion((TypeObject,)),
    StartMotion((TypeObject,)),
    StepSize((TypeObject,TypeVar)),
    StepTime((TypeObject,TypeVar)),
    MoveObj((TypeObject,TypeNum,TypeNum,TypeNum,TypeFlag)),
    MoveObjV((TypeObject,TypeVar,TypeVar,TypeVar,TypeFlag)),
    FollowEgo((TypeObject,TypeNum,TypeFlag)),
    Wander((TypeObject,)),
    NormalMotion((TypeObject,)),
    SetDir((TypeObject,TypeVar)),
    IgnoreBlocks((TypeObject,)),
    ObserveBlocks((TypeObject,)),
    Block((TypeNum,TypeNum,TypeNum,TypeNum)),
    Unblock(()),
    Get((TypeItem,)),
    GetV((TypeVar,)),
    Drop((TypeItem,)),
    LoadSound((TypeNum,)),
    Sound((TypeNum,TypeFlag)),
    StopSound(()),
    Print((TypeMessage,)),
    PrintV((TypeVar,)),
    Display((TypeNum,TypeNum,TypeMessage)),
    DisplayV((TypeVar,TypeVar,TypeVar)),
    ClearLines((TypeNum,TypeNum,TypeNum)),
    TextScreen(()),
    Graphics(()),
    SetCursorChar((TypeMessage,)),
    SetTextAttribute((TypeNum,TypeNum)),
    ShakeScreen((TypeNum,)),
    ConfigureScreen((TypeNum,TypeNum,TypeNum)),
    StatusLineOn(()),
    StatusLineOff(()),
    SetString((TypeString,TypeMessage)),
    GetString((TypeString,TypeMessage,TypeNum,TypeNum,TypeNum)),
    Parse((TypeString,)),
    GetNum((TypeMessage,TypeVar)),
    PreventInput(()),
    AcceptInput(()),
    SetKey((TypeNum,TypeNum,TypeController)),
    AddToPic((TypeNum,TypeNum,TypeNum,TypeNum,TypeNum,TypeNum,TypeNum)),
    AddToPicV((TypeVar,TypeVar,TypeVar,TypeVar,TypeVar,TypeVar,TypeVar)),
    Status(()),
    SaveGame(()),
    RestoreGame(()),
    RestartGame(()),
    ShowObj((TypeNum,)),
    Random((TypeNum,TypeNum,TypeVar)),
    ProgramControl(()),
    PlayerControl(()),
    ObjStatusV((TypeVar,)),
    QuitV0(()),
    QuitV1((TypeNum,)),
    ShowMem(()),
    Pause(()),
    EchoLine(()),
    CancelLine(()),
    InitJoy(()),
    ToggleMonitor(()),
    Version(()),
    SetGameID((TypeMessage,)),
    RepositionTo((TypeObject,TypeNum,TypeNum)),
    RepositionToV((TypeObject,TypeVar,TypeVar)),
    TraceInfo((TypeNum,TypeNum,TypeNum)),
    PrintAtV0((TypeMessage,TypeNum,TypeNum)),
    PrintAtV1((TypeMessage,TypeNum,TypeNum,TypeNum)),
    Goto((TypeGoto,)),
    If((Vec<LogicChange>,TypeGoto)),
}

impl LogicMessages {
    fn new(text_slice: &[u8]) -> Result<LogicMessages,&'static str> {
        // unpack the text data first
        let mut strings:Vec<String> = Vec::new();
        strings.push("".to_string());   // Push [0] "" string, since messages start counting from 1

        let mut iter = text_slice.iter();
        let num_messages = iter.next();
        if num_messages.is_none() {
            return Err("Expected number of messages from input slice")
        }
        let num_messages = num_messages.unwrap();
        let lsb_pos = iter.next();
        if lsb_pos.is_none() {
            return Err("Expected end of messages LSB")
        }
        let lsb_pos = lsb_pos.unwrap();
        let msb_pos = iter.next();
        if msb_pos.is_none() {
            return Err("Expected end of messages MSB");
        }
        let msb_pos = msb_pos.unwrap();
        let position:usize = *msb_pos as usize;
        let position = position<<8;
        let position = position + (*lsb_pos as usize);
        let _end_of_messages = position;
        let decrypt_start_adjust:usize = 2;
        let mut messages:Vec<usize> = Vec::new();
        messages.reserve((*num_messages).into());
        for _ in 0..*num_messages {
            let lsb_pos = iter.next();
            if lsb_pos.is_none() {
                return Err("Expected message LSB for message {m}");
            }
            let lsb_pos=lsb_pos.unwrap();
            let msb_pos = iter.next();
            if msb_pos.is_none() {
                return Err("Expected message MSB for message {m}");
            }
            let msb_pos = msb_pos.unwrap();
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
                    let n = iter.next();
                    if n.is_none() {
                        return Err("Expected message byte for string for message {index}");
                    }
                    let n = n.unwrap();
                    let decrypted = n ^ decrypt_iter.next().unwrap();   // cannot fail as cyclic and static
                    if decrypted == 0 {
                        break;
                    }
                    string = string + &String::from(decrypted as char);
                }
            }

            strings.push(string);
        }

        Ok(LogicMessages { strings })
    }
}

impl LogicResource {
    pub fn new(volume:&Volume, entry: &ResourceDirectoryEntry) -> Result<LogicResource, &'static str> {

        let slice = volume.fetch_data_slice(entry).expect("Expected to be able to fetch slice from entry");
        let mut slice_iter = slice.iter();

        let lsb_pos = slice_iter.next().unwrap();
        let msb_pos = slice_iter.next().unwrap();
        let position:usize = *msb_pos as usize;
        let position = position<<8;
        let position = position + (*lsb_pos as usize);
        let text_start = position;

        let logic_slice = &slice[2..text_start+2];
        let text_slice = &slice[text_start+2..];

        let logic_messages = LogicMessages::new(text_slice).expect("Failed to ");
        let logic_sequence = LogicSequence::new(logic_slice).expect("fsjkdfhksdjf");

        Ok(LogicResource {logic_sequence, logic_messages})
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

    pub fn disassemble(&self,items:&Objects,words:&Words) {

    }
/*
    pub fn disassemble(&self,items:&Objects,words:&Words) {

        let mut iter = self.logic_slice.iter();

        let mut state=LogicState::Action;
        let mut params = [Operands::None; 80];
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
            if params[param_idx] != Operands::None {
                match params[param_idx] {
                    Operands::TypeFlag => { print!("flag:{}",*b); },
                    Operands::TypeNum => { print!("{}",*b); },
                    Operands::TypeVar => { print!("var:{}",*b); },
                    Operands::Object => { print!("obj:{}",*b); },
                    Operands::Item => { print!("item:{}\"{}\"",*b, items.objects[(*b)as usize].name); },
                    Operands::Controller => { print!("ctr:{}",*b); },
                    Operands::Message => { print!("msg:{}\"{}\"",*b,self.logic_messages.strings[(*b) as usize]); },
                    Operands::String => { print!("str:{}",*b); },
                    Operands::WordLSB => { bracket_lsb = (*b).into(); },
                    Operands::WordMSB => { let msb:u16 = (*b).into(); let word:u16=(msb<<8) + bracket_lsb; Self::print_words(words,word); },
                    Operands::Said => { for w in 0..*b as usize { (params[1+w*2+0],params[1+w*2+1])=(Operands::WordLSB,Operands::WordMSB); }},
                    Operands::None => panic!("Should not be reached"),
                }
                let last = params[param_idx];
                params[param_idx]=Operands::None;
                param_idx+=1;
                if params[param_idx] == Operands::None {
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
                        Operands::WordLSB | Operands::Said => {},
                        _ => print!(","),
                    }
                }
            } else {
                match state {
                    LogicState::TestOr => {
                        match b {
                            0xFD => { if expression_continue { print!(" || "); }  print!("!"); expression_continue=false; },
                            0xFC => { print!(")"); state = LogicState::Test },
                            0x12 => { if expression_continue { print!(" || "); } print!("right.posn("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); expression_continue=true; },
                            0x10 => { if expression_continue { print!(" || "); } print!("obj.in.box("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); expression_continue=true; },
                            0x0E => { if expression_continue { print!(" || "); } print!("said("); params[0]=Operands::Said; expression_continue=true; },
                            0x0D => { if expression_continue { print!(" || "); } print!("have.key()"); expression_continue=true; },
                            0x0C => { if expression_continue { print!(" || "); } print!("controller("); params[0]=Operands::Controller; expression_continue=true; },
                            0x0B => { if expression_continue { print!(" || "); } print!("posn("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); expression_continue=true; },
                            0x09 => { if expression_continue { print!(" || "); } print!("has("); params[0]=Operands::Item; expression_continue=true; },
                            0x08 => { if expression_continue { print!(" || "); } print!("issetv("); params[0]=Operands::TypeVar; expression_continue=true; },
                            0x07 => { if expression_continue { print!(" || "); } print!("isset("); params[0]=Operands::TypeFlag; expression_continue=true; },
                            0x06 => { if expression_continue { print!(" || "); } print!("greaterv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); expression_continue=true; },
                            0x05 => { if expression_continue { print!(" || "); } print!("greatern("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); expression_continue=true; },
                            0x04 => { if expression_continue { print!(" || "); } print!("lessv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); expression_continue=true; },
                            0x03 => { if expression_continue { print!(" || "); } print!("lessn("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); expression_continue=true; },
                            0x02 => { if expression_continue { print!(" || "); } print!("equalv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); expression_continue=true; },
                            0x01 => { if expression_continue { print!(" || "); } print!("equaln("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); expression_continue=true; },
                            _ => panic!("Unhandled Test Command Type {:02X}",*b),
                        }
                    },
                    LogicState::Test => {
                        match b {
                            0xFF => { println!(")"); state=LogicState::BracketStart; indent=true;},
                            0xFD => { if expression_continue { print!(" && "); } print!("!"); expression_continue=false; },
                            0xFC => { if expression_continue { print!(" && "); } print!("("); state=LogicState::TestOr; expression_continue=false; },
                            0x12 => { if expression_continue { print!(" && "); } print!("right.posn("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); expression_continue=true; },
                            0x10 => { if expression_continue { print!(" && "); } print!("obj.in.box("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); expression_continue=true; },
                            0x0E => { if expression_continue { print!(" && "); } print!("said("); params[0]=Operands::Said; expression_continue=true; },
                            0x0D => { if expression_continue { print!(" && "); } print!("have.key()"); expression_continue=true; },
                            0x0C => { if expression_continue { print!(" && "); } print!("controller("); params[0]=Operands::Controller; expression_continue=true; },
                            0x0B => { if expression_continue { print!(" && "); } print!("posn("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); expression_continue=true; },
                            0x09 => { if expression_continue { print!(" && "); } print!("has("); params[0]=Operands::Item; expression_continue=true; },
                            0x08 => { if expression_continue { print!(" && "); } print!("issetv("); params[0]=Operands::TypeVar; expression_continue=true; },
                            0x07 => { if expression_continue { print!(" && "); } print!("isset("); params[0]=Operands::TypeFlag; expression_continue=true; },
                            0x06 => { if expression_continue { print!(" && "); } print!("greaterv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); expression_continue=true; },
                            0x05 => { if expression_continue { print!(" && "); } print!("greatern("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); expression_continue=true; },
                            0x04 => { if expression_continue { print!(" && "); } print!("lessv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); expression_continue=true; },
                            0x03 => { if expression_continue { print!(" && "); } print!("lessn("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); expression_continue=true; },
                            0x02 => { if expression_continue { print!(" && "); } print!("equalv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); expression_continue=true; },
                            0x01 => { if expression_continue { print!(" && "); } print!("equaln("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); expression_continue=true; },
                            _ => panic!("Unhandled Test Command Type {:02X}",*b),
                        }
                    },
                    LogicState::Action => {
                        match b {
                            0xFF => { print!("if ("); state=LogicState::Test; expression_continue=false; },
                            0xFE => { print!("goto "); state=LogicState::GotoStart; }
                            0x97 => { print!("print.at("); (params[0],params[1],params[2])=(Operands::Message,Operands::TypeNum,Operands::TypeNum); }, // Version 2.400 onwards has 1 extra numerical argument
                            0x96 => { print!("trace.info("); (params[0],params[1],params[2])=(Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); },
                            0x94 => { print!("reposition.to.v("); (params[0],params[1],params[2])=(Operands::Object,Operands::TypeVar,Operands::TypeVar); },
                            0x93 => { print!("reposition.to("); (params[0],params[1],params[2])=(Operands::Object,Operands::TypeNum,Operands::TypeNum); },
                            0x8F => { print!("set.game.id("); params[0]=Operands::Message;},
                            0x8D => { println!("version();"); indent=true; },
                            0x8C => { println!("toggle.monitor();"); indent=true; },
                            0x8B => { println!("init.joy();"); indent=true; },
                            0x8A => { println!("cancel.line();"); indent=true; },
                            0x89 => { println!("echo.line();"); indent=true; },
                            0x88 => { println!("pause();"); indent=true; },
                            0x87 => { println!("show.mem();"); indent=true; },
                            0x86 => { println!("quit();"); indent=true; },  // Version 2.272 onwards has 1 numerical argument
                            0x85 => { print!("obj.status.v("); params[0]=Operands::TypeVar;},
                            0x84 => { println!("player.control();"); indent=true; },
                            0x83 => { println!("program.control();"); indent=true; },
                            0x82 => { print!("random("); (params[0],params[1],params[2])=(Operands::TypeNum,Operands::TypeNum,Operands::TypeVar); },
                            0x81 => { print!("show.obj("); params[0]=Operands::TypeNum;},
                            0x80 => { println!("restart.game();"); indent=true; },
                            0x7E => { println!("restore.game();"); indent=true; },
                            0x7D => { println!("save.game();"); indent=true; },
                            0x7C => { println!("status();"); indent=true; },
                            0x7B => { print!("add.to.pic.v("); (params[0],params[1],params[2],params[3],params[4],params[5],params[6])=(Operands::TypeVar,Operands::TypeVar,Operands::TypeVar,Operands::TypeVar,Operands::TypeVar,Operands::TypeVar,Operands::TypeVar); },
                            0x7A => { print!("add.to.pic("); (params[0],params[1],params[2],params[3],params[4],params[5],params[6])=(Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); },
                            0x79 => { print!("set.key("); (params[0],params[1],params[2])=(Operands::TypeNum,Operands::TypeNum,Operands::Controller); },
                            0x78 => { println!("accept.input();"); indent=true; },
                            0x77 => { println!("prevent.input();"); indent=true; },
                            0x76 => { print!("get.num("); (params[0],params[1])=(Operands::Message,Operands::TypeVar); },
                            0x75 => { print!("parse("); params[0]=Operands::String;},
                            0x73 => { print!("get.string("); (params[0],params[1],params[2],params[3],params[4])=(Operands::String,Operands::Message,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); },
                            0x72 => { print!("set.string("); (params[0],params[1])=(Operands::String,Operands::Message); },
                            0x71 => { println!("status.line.off();"); indent=true; },
                            0x70 => { println!("status.line.on();"); indent=true; },
                            0x6F => { print!("configure.screen("); (params[0],params[1],params[2])=(Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); },
                            0x6E => { print!("shake.screen("); params[0]=Operands::TypeNum;},
                            0x6D => { print!("set.text.attribute("); (params[0],params[1])=(Operands::TypeNum,Operands::TypeNum); },
                            0x6C => { print!("set.cursor.char("); params[0]=Operands::Message;},
                            0x6B => { println!("graphics();"); indent=true; },
                            0x6A => { println!("text.screen();"); indent=true; },
                            0x69 => { print!("clear.lines("); (params[0],params[1],params[2])=(Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); },
                            0x68 => { print!("display.v("); (params[0],params[1],params[2])=(Operands::TypeVar,Operands::TypeVar,Operands::TypeVar); },
                            0x67 => { print!("display("); (params[0],params[1],params[2])=(Operands::TypeNum,Operands::TypeNum,Operands::Message); },
                            0x66 => { print!("print.v("); params[0]=Operands::TypeVar;},
                            0x65 => { print!("print("); params[0]=Operands::Message;},
                            0x64 => { println!("stop.sound();"); indent=true; },
                            0x63 => { print!("sound("); (params[0],params[1])=(Operands::TypeNum,Operands::TypeFlag); },
                            0x62 => { print!("load.sound("); params[0]=Operands::TypeNum;},
                            0x5E => { print!("drop("); params[0]=Operands::Item;},
                            0x5D => { print!("get.v("); params[0]=Operands::TypeVar;},
                            0x5C => { print!("get("); params[0]=Operands::Item;},
                            0x5B => { println!("unblock();"); indent=true; },
                            0x5A => { print!("block("); (params[0],params[1],params[2],params[3])=(Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum); },
                            0x59 => { print!("observe.blocks("); params[0]=Operands::Object;},
                            0x58 => { print!("ignore.blocks("); params[0]=Operands::Object;},
                            0x56 => { print!("set.dir("); (params[0],params[1])=(Operands::Object,Operands::TypeVar);},
                            0x55 => { print!("normal.motion("); params[0]=Operands::Object;},
                            0x54 => { print!("wander("); params[0]=Operands::Object;},
                            0x53 => { print!("follow.ego("); (params[0],params[1],params[2])=(Operands::Object,Operands::TypeNum,Operands::TypeFlag); },
                            0x52 => { print!("move.obj.v("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeVar,Operands::TypeVar,Operands::TypeNum,Operands::TypeFlag); },
                            0x51 => { print!("move.obj("); (params[0],params[1],params[2],params[3],params[4])=(Operands::Object,Operands::TypeNum,Operands::TypeNum,Operands::TypeNum,Operands::TypeFlag); },
                            0x50 => { print!("step.time("); (params[0],params[1])=(Operands::Object,Operands::TypeVar);},
                            0x4F => { print!("step.size("); (params[0],params[1])=(Operands::Object,Operands::TypeVar);},
                            0x4E => { print!("start.motion("); params[0]=Operands::Object;},
                            0x4D => { print!("stop.motion("); params[0]=Operands::Object;},
                            0x4C => { print!("cycle.time("); (params[0],params[1])=(Operands::Object,Operands::TypeVar);},
                            0x4B => { print!("reverse.loop("); (params[0],params[1])=(Operands::Object,Operands::TypeFlag);},
                            0x49 => { print!("end.of.loop("); (params[0],params[1])=(Operands::Object,Operands::TypeFlag);},
                            0x47 => { print!("start.cycling("); params[0]=Operands::Object;},
                            0x46 => { print!("stop.cycling("); params[0]=Operands::Object;},
                            0x45 => { print!("distance("); (params[0],params[1],params[2])=(Operands::Object,Operands::Object,Operands::TypeVar); },
                            0x44 => { print!("observe.objs("); params[0]=Operands::Object;},
                            0x43 => { print!("ignore.objs("); params[0]=Operands::Object;},
                            0x41 => { print!("object.on.land("); params[0]=Operands::Object;},
                            0x3F => { print!("set.horizon("); params[0]=Operands::TypeNum;},
                            0x3E => { print!("observe.horizon("); params[0]=Operands::Object;},
                            0x3D => { print!("ignore.horizon("); params[0]=Operands::Object;},
                            0x3C => { print!("force.update("); params[0]=Operands::Object;},
                            0x3B => { print!("start.update("); params[0]=Operands::Object;},
                            0x3A => { print!("stop.update("); params[0]=Operands::Object;},
                            0x39 => { print!("get.priority("); (params[0],params[1])=(Operands::Object,Operands::TypeVar);},
                            0x38 => { print!("release.priority("); params[0]=Operands::Object;},
                            0x36 => { print!("set.priority("); (params[0],params[1])=(Operands::Object,Operands::TypeNum);},
                            //0x35 => { print!("number.of.loops("); (params[0],params[1])=(Params::Object,Params::TypeVar);},
                            0x34 => { print!("current.view("); (params[0],params[1])=(Operands::Object,Operands::TypeVar); },
                            0x33 => { print!("current.loop("); (params[0],params[1])=(Operands::Object,Operands::TypeVar); },
                            0x32 => { print!("current.cel("); (params[0],params[1])=(Operands::Object,Operands::TypeVar); },
                            0x31 => { print!("last.cel("); (params[0],params[1])=(Operands::Object,Operands::TypeVar); },
                            0x30 => { print!("set.cel.v("); (params[0],params[1])=(Operands::Object,Operands::TypeVar); },
                            0x2F => { print!("set.cel("); (params[0],params[1])=(Operands::Object,Operands::TypeNum); },
                            0x2E => { print!("release.loop("); params[0]=Operands::Object; },
                            0x2D => { print!("fix.loop("); params[0]=Operands::Object; },
                            0x2C => { print!("set.loop.v("); (params[0],params[1])=(Operands::Object,Operands::TypeVar); },
                            0x2B => { print!("set.loop("); (params[0],params[1])=(Operands::Object,Operands::TypeNum); },
                            0x2A => { print!("set.view.v("); (params[0],params[1])=(Operands::Object,Operands::TypeVar); },
                            0x29 => { print!("set.view("); (params[0],params[1])=(Operands::Object,Operands::TypeNum); },
                            0x28 => { print!("reposition("); (params[0],params[1],params[2])=(Operands::Object,Operands::TypeVar,Operands::TypeVar); },
                            0x27 => { print!("get.posn("); (params[0],params[1],params[2])=(Operands::Object,Operands::TypeVar,Operands::TypeVar); },
                            0x26 => { print!("position("); (params[0],params[1],params[2])=(Operands::Object,Operands::TypeVar,Operands::TypeVar); },
                            0x25 => { print!("position("); (params[0],params[1],params[2])=(Operands::Object,Operands::TypeNum,Operands::TypeNum); },
                            0x24 => { print!("erase("); params[0]=Operands::Object; },
                            0x23 => { print!("draw("); params[0]=Operands::Object; },
                            0x22 => { println!("unanimate.all();"); indent=true; },
                            0x21 => { print!("animate.obj("); params[0]=Operands::Object; },
                            0x20 => { print!("discard.view("); params[0]=Operands::TypeNum; },
                            0x1F => { print!("load.view.v("); params[0]=Operands::TypeVar; },
                            0x1E => { print!("load.view("); params[0]=Operands::TypeNum; },
                            0x1D => { println!("show.pri.screen();"); indent=true; },
                            0x1B => { print!("discard.pic("); params[0]=Operands::TypeVar; },
                            0x1A => { println!("show.pic();"); indent=true; },
                            0x19 => { print!("draw.pic("); params[0]=Operands::TypeVar; },
                            0x18 => { print!("load.pic("); params[0]=Operands::TypeVar; },
                            0x17 => { print!("call.v("); params[0]=Operands::TypeVar; },
                            0x16 => { print!("call("); params[0]=Operands::TypeNum; },
                            0x14 => { print!("load.logics("); params[0]=Operands::TypeNum; },
                            0x13 => { print!("new.room.v("); params[0]=Operands::TypeVar; },
                            0x12 => { print!("new.room("); params[0]=Operands::TypeNum; },
                            0x10 => { print!("reset.v("); params[0]=Operands::TypeVar; },
                            0x0F => { print!("set.v("); params[0]=Operands::TypeVar; },
                            0x0E => { print!("toggle("); params[0]=Operands::TypeFlag; },
                            0x0D => { print!("reset("); params[0]=Operands::TypeFlag; },
                            0x0C => { print!("set("); params[0]=Operands::TypeFlag; },
                            0x0B => { print!("lindirectn("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); },
                            0x0A => { print!("rindirect("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); },
                            0x09 => { print!("lindirectv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); },
                            0x08 => { print!("subv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); },
                            0x07 => { print!("subn("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); },
                            0x06 => { print!("addv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); },
                            0x05 => { print!("addn("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); },
                            0x04 => { print!("assignv("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeVar); },
                            0x03 => { print!("assignn("); (params[0],params[1])=(Operands::TypeVar,Operands::TypeNum); },
                            0x02 => { print!("decrement("); params[0]=Operands::TypeVar; },
                            0x01 => { print!("increment("); params[0]=Operands::TypeVar; },
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
    }*/
}

impl LogicSequence {

    fn read_little_endian_i16(iter:&mut std::slice::Iter<u8>) -> Result<i16, &'static str> {
        let lsb = iter.next();
        if lsb.is_none() {
            return Err("Expected LSB of U16, but reached end of iterator");
        }
        let lsb:i16 = (*lsb.unwrap()).into();
        let msb = iter.next();
        if msb.is_none() {
            return Err("Expected MSB of U16, but reached end of iterator");
        }
        let msb:i16 = (*msb.unwrap()).into();
        return Ok(((msb<<8)+lsb).into());
    }

    fn read_little_endian_u16(iter:&mut std::slice::Iter<u8>) -> Result<u16, &'static str> {
        let lsb = iter.next();
        if lsb.is_none() {
            return Err("Expected LSB of U16, but reached end of iterator");
        }
        let lsb:u16 = (*lsb.unwrap()).into();
        let msb = iter.next();
        if msb.is_none() {
            return Err("Expected MSB of U16, but reached end of iterator");
        }
        let msb:u16 = (*msb.unwrap()).into();
        return Ok(((msb<<8)+lsb).into());
    }

    fn parse_goto(iter:&mut std::slice::Iter<u8>) -> Result<TypeGoto, &'static str> {
        return Ok(Self::read_little_endian_i16(iter)?.into());
    }

    fn parse_message(iter:&mut std::slice::Iter<u8>) -> Result<TypeMessage, &'static str> {
        let m = iter.next();
        if m.is_none() {
            return Err("Expected Message, but reached end of iterator");
        }
        return Ok((*m.unwrap()).into());
    }

    fn parse_string(iter:&mut std::slice::Iter<u8>) -> Result<TypeString, &'static str> {
        let s = iter.next();
        if s.is_none() {
            return Err("Expected String, but reached end of iterator");
        }
        return Ok((*s.unwrap()).into());
    }

    fn parse_object(iter:&mut std::slice::Iter<u8>) -> Result<TypeObject, &'static str> {
        let o = iter.next();
        if o.is_none() {
            return Err("Expected Object, but reached end of iterator");
        }
        return Ok((*o.unwrap()).into());
    }

    fn parse_controller(iter:&mut std::slice::Iter<u8>) -> Result<TypeController, &'static str> {
        let c = iter.next();
        if c.is_none() {
            return Err("Expected Controller, but reached end of iterator");
        }
        return Ok((*c.unwrap()).into());
    }

    fn parse_item(iter:&mut std::slice::Iter<u8>) -> Result<TypeItem, &'static str> {
        let i = iter.next();
        if i.is_none() {
            return Err("Expected Item, but reached end of iterator");
        }
        return Ok((*i.unwrap()).into());
    }

    fn parse_flag(iter:&mut std::slice::Iter<u8>) -> Result<TypeFlag, &'static str> {
        let f = iter.next();
        if f.is_none() {
            return Err("Expected TypeFlag, but reached end of iterator");
        }
        return Ok((*f.unwrap()).into());
    }

    fn parse_var(iter:&mut std::slice::Iter<u8>) -> Result<TypeVar, &'static str> {
        let v = iter.next();
        if v.is_none() {
            return Err("Expected TypeVariable, but reached end of iterator");
        }
        return Ok((*v.unwrap()).into());
    }
    
    fn parse_num(iter:&mut std::slice::Iter<u8>) -> Result<TypeNum, &'static str> {
        let n = iter.next();
        if n.is_none() {
            return Err("Expected TypeNumber, but reached end of iterator");
        }
        return Ok((*n.unwrap()).into());
    }

    fn parse_word(iter:&mut std::slice::Iter<u8>) -> Result<TypeWord, &'static str> {
        return Ok(Self::read_little_endian_u16(iter)?.into());
    }

    fn parse_said(iter:&mut std::slice::Iter<u8>) -> Result<Vec<TypeWord>, &'static str> {
        let cnt = iter.next();
        if cnt.is_none() {
            return Err("Expected Cnt of arguments, but reached end of iterator");
        }
        let cnt:usize=(*cnt.unwrap()).into();
        let mut words:Vec<TypeWord> = Vec::new();
        words.reserve(cnt);
        for _a in 0..cnt {
            words.push(Self::parse_word(iter)?);
        }
        return Ok(words);
    }

    fn parse_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_num(iter)?));
    }

    fn parse_num_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeFlag), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_flag(iter)?));
    }
    
    fn parse_var_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeNum), &'static str> {
        return Ok((Self::parse_var(iter)?,Self::parse_num(iter)?));
    }
    
    fn parse_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeVar), &'static str> {
        return Ok((Self::parse_var(iter)?,Self::parse_var(iter)?));
    }
    
    fn parse_object_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_num(iter)?));
    }
    
    fn parse_object_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeVar), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_var(iter)?));
    }

    fn parse_object_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeFlag), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_flag(iter)?));
    }

    fn parse_string_message(iter:&mut std::slice::Iter<u8>) -> Result<(TypeString,TypeMessage), &'static str> {
        return Ok((Self::parse_string(iter)?,Self::parse_message(iter)?));
    }

    fn parse_message_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeMessage,TypeVar), &'static str> {
        return Ok((Self::parse_message(iter)?,Self::parse_var(iter)?));
    }

    fn parse_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?));
    }

    fn parse_num_num_message(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeMessage), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_message(iter)?));
    }

    fn parse_num_num_controller(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeController), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_controller(iter)?));
    }

    fn parse_num_num_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeVar), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_var(iter)?));
    }

    fn parse_var_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeVar,TypeVar), &'static str> {
        return Ok((Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?));
    }

    fn parse_object_object_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeObject,TypeVar), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_object(iter)?,Self::parse_var(iter)?));
    }

    fn parse_message_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeMessage,TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_message(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?));
    }

    fn parse_object_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeVar,TypeVar), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?));
    }

    fn parse_object_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?));
    }

    fn parse_object_num_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeFlag), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_flag(iter)?));
    }

    fn parse_num_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?));
    }
    
    fn parse_object_num_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeNum,TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?));
    }
    
    fn parse_object_num_num_num_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeNum,TypeNum,TypeFlag), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_flag(iter)?));
    }
    
    fn parse_object_var_var_var_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeVar,TypeVar,TypeVar,TypeFlag), &'static str> {
        return Ok((Self::parse_object(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_flag(iter)?));
    }
    
    fn parse_string_message_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeString,TypeMessage,TypeNum,TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_string(iter)?,Self::parse_message(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?));
    }

    fn parse_num_num_num_num_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeNum,TypeNum,TypeNum,TypeNum,TypeNum), &'static str> {
        return Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?));
    }
    
    fn parse_var_var_var_var_var_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeVar,TypeVar,TypeVar,TypeVar,TypeVar,TypeVar), &'static str> {
        return Ok((Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?));
    }
    
    fn parse_condition_with_code(iter:&mut std::slice::Iter<u8>, code:u8) -> Result<LogicOperation, &'static str> {
        return match code {
            0x12 => Ok(LogicOperation::RightPosN(Self::parse_object_num_num_num_num(iter)?)),
            0x10 => Ok(LogicOperation::ObjInBox(Self::parse_object_num_num_num_num(iter)?)),
            0x0E => Ok(LogicOperation::Said((Self::parse_said(iter)?,))),
            0x0D => Ok(LogicOperation::HaveKey(())),
            0x0C => Ok(LogicOperation::Controller((Self::parse_controller(iter)?,))),
            0x0B => Ok(LogicOperation::PosN(Self::parse_object_num_num_num_num(iter)?)),
            0x09 => Ok(LogicOperation::Has((Self::parse_item(iter)?,))),
            0x08 => Ok(LogicOperation::IsSetV((Self::parse_var(iter)?,))),
            0x07 => Ok(LogicOperation::IsSet((Self::parse_flag(iter)?,))),
            0x06 => Ok(LogicOperation::GreaterV(Self::parse_var_var(iter)?)),
            0x05 => Ok(LogicOperation::GreaterN(Self::parse_var_num(iter)?)),
            0x04 => Ok(LogicOperation::LessV(Self::parse_var_var(iter)?)),
            0x03 => Ok(LogicOperation::LessN(Self::parse_var_num(iter)?)),
            0x02 => Ok(LogicOperation::EqualV(Self::parse_var_var(iter)?)),
            0x01 => Ok(LogicOperation::EqualN(Self::parse_var_num(iter)?)),
            _ => Err("Unexpected LogicOperation {code:02X}"),
        }
    }

    fn parse_condition(iter:&mut std::slice::Iter<u8>) -> Result<LogicOperation, &'static str> {
        let code = iter.next();
        if code.is_none() {
            return Err("Expected condition code, but reached end of iterator");
        }
        return Self::parse_condition_with_code(iter,*(code.unwrap()));
    }

    fn parse_or(iter:&mut std::slice::Iter<u8>) -> Result<Vec<LogicChange>, &'static str> {
        let mut or:Vec<LogicChange> = Vec::new();
        loop {
            let b = iter.next();
            if b.is_none() {
                return Err("Expected another logic operation, but reached end of iterator");
            }
            let b = *(b.unwrap());
            match b {
                0xFC => break,
                0xFD => or.push(LogicChange::Not((Self::parse_condition(iter)?,))),
                _ => or.push(LogicChange::Normal((Self::parse_condition_with_code(iter, b)?,))),
            }
        }
        return Ok(or);
    }

    fn parse_vlogic_change_goto(iter:&mut std::slice::Iter<u8>) -> Result<(Vec<LogicChange>,TypeGoto), &'static str> {

        // First off read all the tests
        let mut conditions:Vec<LogicChange> = Vec::new();
        while let Some(b) = iter.next() {
            match b {
                0xFF => break,
                0xFD => conditions.push(LogicChange::Not((Self::parse_condition(iter)?,))),
                0xFC => conditions.push(LogicChange::Or((Self::parse_or(iter)?,))),
                _ => conditions.push(LogicChange::Normal((Self::parse_condition_with_code(iter, *b)?,))),
            }
        }

        // Finally read the goto values
        let pos = Self::parse_goto(iter)?;

        return Ok((conditions,pos));
    }

    fn new(logic_slice: &[u8]) -> Result<LogicSequence,&'static str> {

        let mut iter = logic_slice.iter();

        let mut operations:Vec<ActionOperation> = Vec::new();
        let mut offsets:HashMap<TypeGoto, usize>=HashMap::new();
        let mut offsets_rev:HashMap<usize, TypeGoto>=HashMap::new();
        let initial_size = logic_slice.len();

        operations.reserve(initial_size);  // over allocate then shrink to fit at end of process (over allocates,because there are operands mixed into the stream)
        offsets.reserve(initial_size);
        offsets_rev.reserve(initial_size);

        while let Some(b) = iter.next()
        {
            let program_position = initial_size - iter.as_slice().len() -1;
            offsets.insert((program_position as i16).into(), operations.len());
            offsets_rev.insert(operations.len(),(program_position as i16).into());
            match b {
                0xFF => operations.push(ActionOperation::If(Self::parse_vlogic_change_goto(&mut iter)?)),
                0xFE => operations.push(ActionOperation::Goto((Self::parse_goto(&mut iter)?,))),
                0x97 => operations.push(ActionOperation::PrintAtV0(Self::parse_message_num_num(&mut iter)?)),
                0x96 => operations.push(ActionOperation::TraceInfo(Self::parse_num_num_num(&mut iter)?)),
                0x94 => operations.push(ActionOperation::RepositionToV(Self::parse_object_var_var(&mut iter)?)),
                0x93 => operations.push(ActionOperation::RepositionTo(Self::parse_object_num_num(&mut iter)?)),
                0x8F => operations.push(ActionOperation::SetGameID((Self::parse_message(&mut iter)?,))),
                0x8D => operations.push(ActionOperation::Version(())),
                0x8C => operations.push(ActionOperation::ToggleMonitor(())),
                0x8B => operations.push(ActionOperation::InitJoy(())),
                0x8A => operations.push(ActionOperation::CancelLine(())),
                0x89 => operations.push(ActionOperation::EchoLine(())),
                0x88 => operations.push(ActionOperation::Pause(())),
                0x87 => operations.push(ActionOperation::ShowMem(())),
                0x86 => operations.push(ActionOperation::QuitV0(())),
                0x85 => operations.push(ActionOperation::ObjStatusV((Self::parse_var(&mut iter)?,))),
                0x84 => operations.push(ActionOperation::PlayerControl(())),
                0x83 => operations.push(ActionOperation::ProgramControl(())),
                0x82 => operations.push(ActionOperation::Random(Self::parse_num_num_var(&mut iter)?)),
                0x81 => operations.push(ActionOperation::ShowObj((Self::parse_num(&mut iter)?,))),
                0x80 => operations.push(ActionOperation::RestartGame(())),
                0x7E => operations.push(ActionOperation::RestoreGame(())),
                0x7D => operations.push(ActionOperation::SaveGame(())),
                0x7C => operations.push(ActionOperation::Status(())),
                0x7B => operations.push(ActionOperation::AddToPicV(Self::parse_var_var_var_var_var_var_var(&mut iter)?)),
                0x7A => operations.push(ActionOperation::AddToPic(Self::parse_num_num_num_num_num_num_num(&mut iter)?)),
                0x79 => operations.push(ActionOperation::SetKey(Self::parse_num_num_controller(&mut iter)?)),
                0x78 => operations.push(ActionOperation::AcceptInput(())),
                0x77 => operations.push(ActionOperation::PreventInput(())),
                0x76 => operations.push(ActionOperation::GetNum(Self::parse_message_var(&mut iter)?)),
                0x75 => operations.push(ActionOperation::Parse((Self::parse_string(&mut iter)?,))),
                0x73 => operations.push(ActionOperation::GetString(Self::parse_string_message_num_num_num(&mut iter)?)),
                0x72 => operations.push(ActionOperation::SetString(Self::parse_string_message(&mut iter)?)),
                0x71 => operations.push(ActionOperation::StatusLineOff(())),
                0x70 => operations.push(ActionOperation::StatusLineOn(())),
                0x6F => operations.push(ActionOperation::ConfigureScreen(Self::parse_num_num_num(&mut iter)?)),
                0x6E => operations.push(ActionOperation::ShakeScreen((Self::parse_num(&mut iter)?,))),
                0x6D => operations.push(ActionOperation::SetTextAttribute(Self::parse_num_num(&mut iter)?)),
                0x6C => operations.push(ActionOperation::SetCursorChar((Self::parse_message(&mut iter)?,))),
                0x6B => operations.push(ActionOperation::Graphics(())),
                0x6A => operations.push(ActionOperation::TextScreen(())),
                0x69 => operations.push(ActionOperation::ClearLines(Self::parse_num_num_num(&mut iter)?)),
                0x68 => operations.push(ActionOperation::DisplayV(Self::parse_var_var_var(&mut iter)?)),
                0x67 => operations.push(ActionOperation::Display(Self::parse_num_num_message(&mut iter)?)),
                0x66 => operations.push(ActionOperation::PrintV((Self::parse_var(&mut iter)?,))),
                0x65 => operations.push(ActionOperation::Print((Self::parse_message(&mut iter)?,))),
                0x64 => operations.push(ActionOperation::StopSound(())),
                0x63 => operations.push(ActionOperation::Sound(Self::parse_num_flag(&mut iter)?)),
                0x62 => operations.push(ActionOperation::LoadSound((Self::parse_num(&mut iter)?,))),
                0x5E => operations.push(ActionOperation::Drop((Self::parse_item(&mut iter)?,))),
                0x5D => operations.push(ActionOperation::GetV((Self::parse_var(&mut iter)?,))),
                0x5C => operations.push(ActionOperation::Get((Self::parse_item(&mut iter)?,))),
                0x5B => operations.push(ActionOperation::Unblock(())),
                0x5A => operations.push(ActionOperation::Block(Self::parse_num_num_num_num(&mut iter)?)),
                0x59 => operations.push(ActionOperation::ObserveBlocks((Self::parse_object(&mut iter)?,))),
                0x58 => operations.push(ActionOperation::IgnoreBlocks((Self::parse_object(&mut iter)?,))),
                0x56 => operations.push(ActionOperation::SetDir(Self::parse_object_var(&mut iter)?)),
                0x55 => operations.push(ActionOperation::NormalMotion((Self::parse_object(&mut iter)?,))),
                0x54 => operations.push(ActionOperation::Wander((Self::parse_object(&mut iter)?,))),
                0x53 => operations.push(ActionOperation::FollowEgo(Self::parse_object_num_flag(&mut iter)?)),
                0x52 => operations.push(ActionOperation::MoveObjV(Self::parse_object_var_var_var_flag(&mut iter)?)),
                0x51 => operations.push(ActionOperation::MoveObj(Self::parse_object_num_num_num_flag(&mut iter)?)),
                0x50 => operations.push(ActionOperation::StepTime(Self::parse_object_var(&mut iter)?)),
                0x4F => operations.push(ActionOperation::StepSize(Self::parse_object_var(&mut iter)?)),
                0x4E => operations.push(ActionOperation::StartMotion((Self::parse_object(&mut iter)?,))),
                0x4D => operations.push(ActionOperation::StopMotion((Self::parse_object(&mut iter)?,))),
                0x4C => operations.push(ActionOperation::CycleTime(Self::parse_object_var(&mut iter)?)),
                0x4B => operations.push(ActionOperation::ReverseLoop(Self::parse_object_flag(&mut iter)?)),
                0x49 => operations.push(ActionOperation::EndOfLoop(Self::parse_object_flag(&mut iter)?)),
                0x47 => operations.push(ActionOperation::StartCycling((Self::parse_object(&mut iter)?,))),
                0x46 => operations.push(ActionOperation::StopCycling((Self::parse_object(&mut iter)?,))),
                0x45 => operations.push(ActionOperation::Distance(Self::parse_object_object_var(&mut iter)?)),
                0x44 => operations.push(ActionOperation::ObserveObjs((Self::parse_object(&mut iter)?,))),
                0x43 => operations.push(ActionOperation::IgnoreObjs((Self::parse_object(&mut iter)?,))),
                0x41 => operations.push(ActionOperation::ObjectOnLand((Self::parse_object(&mut iter)?,))),
                //0x40 => operations.push(ActionOperation::ObjectOnWater((Self::parse_object(&mut iter)?,))),
                0x3F => operations.push(ActionOperation::SetHorizon((Self::parse_num(&mut iter)?,))),
                0x3E => operations.push(ActionOperation::ObserveHorizon((Self::parse_object(&mut iter)?,))),
                0x3D => operations.push(ActionOperation::IgnoreHorizon((Self::parse_object(&mut iter)?,))),
                0x3C => operations.push(ActionOperation::ForceUpdate((Self::parse_object(&mut iter)?,))),
                0x3B => operations.push(ActionOperation::StartUpdate((Self::parse_object(&mut iter)?,))),
                0x3A => operations.push(ActionOperation::StopUpdate((Self::parse_object(&mut iter)?,))),
                0x39 => operations.push(ActionOperation::GetPriority(Self::parse_object_var(&mut iter)?)),
                0x38 => operations.push(ActionOperation::ReleasePriority((Self::parse_object(&mut iter)?,))),
                //0x37 => operations.push(ActionOperation::SetPriorityV(Self::parse_object_var(&mut iter)?)),
                0x36 => operations.push(ActionOperation::SetPriority(Self::parse_object_num(&mut iter)?)),
                0x34 => operations.push(ActionOperation::CurrentView(Self::parse_object_var(&mut iter)?)),
                0x33 => operations.push(ActionOperation::CurrentLoop(Self::parse_object_var(&mut iter)?)),
                0x32 => operations.push(ActionOperation::CurrentCel(Self::parse_object_var(&mut iter)?)),
                0x31 => operations.push(ActionOperation::LastCel(Self::parse_object_var(&mut iter)?)),
                0x30 => operations.push(ActionOperation::SetCelV(Self::parse_object_var(&mut iter)?)),
                0x2F => operations.push(ActionOperation::SetCel(Self::parse_object_num(&mut iter)?,)),
                0x2E => operations.push(ActionOperation::ReleaseLoop((Self::parse_object(&mut iter)?,))),
                0x2D => operations.push(ActionOperation::FixLoop((Self::parse_object(&mut iter)?,))),
                0x2C => operations.push(ActionOperation::SetLoopV(Self::parse_object_var(&mut iter)?)),
                0x2B => operations.push(ActionOperation::SetLoop(Self::parse_object_num(&mut iter)?)),
                0x2A => operations.push(ActionOperation::SetViewV(Self::parse_object_var(&mut iter)?)),
                0x29 => operations.push(ActionOperation::SetView(Self::parse_object_num(&mut iter)?)),
                0x28 => operations.push(ActionOperation::Reposition(Self::parse_object_var_var(&mut iter)?)),
                0x27 => operations.push(ActionOperation::GetPosN(Self::parse_object_var_var(&mut iter)?)),
                0x26 => operations.push(ActionOperation::PositionV(Self::parse_object_var_var(&mut iter)?)),
                0x25 => operations.push(ActionOperation::Position(Self::parse_object_num_num(&mut iter)?)),
                0x24 => operations.push(ActionOperation::Erase((Self::parse_object(&mut iter)?,))),
                0x23 => operations.push(ActionOperation::Draw((Self::parse_object(&mut iter)?,))),
                0x22 => operations.push(ActionOperation::UnanimateAll(())),
                0x21 => operations.push(ActionOperation::AnimateObj((Self::parse_object(&mut iter)?,))),
                0x20 => operations.push(ActionOperation::DiscardView((Self::parse_num(&mut iter)?,))),
                0x1F => operations.push(ActionOperation::LoadViewV((Self::parse_var(&mut iter)?,))),
                0x1E => operations.push(ActionOperation::LoadView((Self::parse_num(&mut iter)?,))),
                0x1D => operations.push(ActionOperation::ShowPriScreen(())),
                0x1B => operations.push(ActionOperation::DiscardPic((Self::parse_var(&mut iter)?,))),
                0x1A => operations.push(ActionOperation::ShowPic(())),
                0x19 => operations.push(ActionOperation::DrawPic((Self::parse_var(&mut iter)?,))),
                0x18 => operations.push(ActionOperation::LoadPic((Self::parse_var(&mut iter)?,))),
                0x17 => operations.push(ActionOperation::CallV((Self::parse_var(&mut iter)?,))),
                0x16 => operations.push(ActionOperation::Call((Self::parse_num(&mut iter)?,))),
                0x14 => operations.push(ActionOperation::LoadLogic((Self::parse_num(&mut iter)?,))),
                0x13 => operations.push(ActionOperation::NewRoomV((Self::parse_var(&mut iter)?,))),
                0x12 => operations.push(ActionOperation::NewRoom((Self::parse_num(&mut iter)?,))),
                0x10 => operations.push(ActionOperation::ResetV((Self::parse_var(&mut iter)?,))),
                0x0F => operations.push(ActionOperation::SetV((Self::parse_var(&mut iter)?,))),
                0x0E => operations.push(ActionOperation::Toggle((Self::parse_flag(&mut iter)?,))),
                0x0D => operations.push(ActionOperation::Reset((Self::parse_flag(&mut iter)?,))),
                0x0C => operations.push(ActionOperation::Set((Self::parse_flag(&mut iter)?,))),
                0x0B => operations.push(ActionOperation::LIndirectN(Self::parse_var_num(&mut iter)?)),
                0x0A => operations.push(ActionOperation::RIndirect(Self::parse_var_var(&mut iter)?)),
                0x09 => operations.push(ActionOperation::LIndirectV(Self::parse_var_var(&mut iter)?)),
                0x08 => operations.push(ActionOperation::SubV(Self::parse_var_var(&mut iter)?)),
                0x07 => operations.push(ActionOperation::SubN(Self::parse_var_num(&mut iter)?)),
                0x06 => operations.push(ActionOperation::AddV(Self::parse_var_var(&mut iter)?)),
                0x05 => operations.push(ActionOperation::AddN(Self::parse_var_num(&mut iter)?)),
                0x04 => operations.push(ActionOperation::AssignV(Self::parse_var_var(&mut iter)?)),
                0x03 => operations.push(ActionOperation::AssignN(Self::parse_var_num(&mut iter)?)),
                0x02 => operations.push(ActionOperation::Decrement((Self::parse_var(&mut iter)?,))),
                0x01 => operations.push(ActionOperation::Increment((Self::parse_var(&mut iter)?,))),
                0x00 => operations.push(ActionOperation::Return(())),
                _ => {panic!("Unimplemented action {b:02X}");}
            }
        }

        let mut labels:HashMap<TypeGoto, (bool,usize)>=HashMap::new();
        labels.reserve(operations.len());
        for (index,op) in operations.iter().enumerate() {
            match op {
                ActionOperation::Goto((g,)) => {
                    let base_offset=offsets_rev[&(index+1)];
                    let destination:TypeGoto = base_offset+*g;
                    labels.insert(*g, (true,offsets[&destination]));
                },
                ActionOperation::If((_,g)) => {
                    let base_offset=offsets_rev[&(index+1)];
                    let destination:TypeGoto = base_offset+*g;
                    labels.insert(*g, (false,offsets[&destination]));
                }
                _ => {},
            }
        }

        operations.shrink_to_fit();
        labels.shrink_to_fit();
        return Ok(LogicSequence { operations, labels });
    }
}