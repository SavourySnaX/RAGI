use std::{collections::{HashMap, VecDeque}, hash::Hash, ops};

use dir_resource::{ResourceDirectoryEntry, ResourcesVersion, ResourceCompression};

use objects::Objects;
use volume::{Volume, VolumeCache};
use words::Words;

use strum_macros::IntoStaticStr;

pub const SCREEN_WIDTH_USIZE:usize = 320;
pub const SCREEN_HEIGHT_USIZE:usize = 200;


pub struct LogicResource {
    logic_sequence:LogicSequence,
    logic_messages:LogicMessages,
}

pub struct LogicMessages {
    pub strings:Vec<String>,
}

pub struct LogicOperation {
    pub action:ActionOperation,
    pub address:TypeGoto,
}

pub struct Label {
    is_goto_destination:bool,
    if_destination_cnt:u16,
    operation_offset:usize,
}

pub struct LogicSequence {
    operations:Vec<LogicOperation>,
    labels:HashMap<TypeGoto,Label>,
}

#[duplicate_item(name; [TypeFlag]; [TypeNum]; [TypeVar]; [TypeObject]; [TypeController]; [TypeMessage]; [TypeString]; [TypeItem])]
#[derive(Clone,Copy,Debug,PartialEq)]
#[allow(dead_code)]
pub struct name {
    value:u8,
}


#[derive(Clone,Copy,Debug)]
pub struct TypeWord {
    value:u16,
}

#[derive(Clone,Copy,Eq,Hash,PartialEq,Debug)]
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

#[duplicate_item(name; [TypeFlag]; [TypeNum]; [TypeVar]; [TypeObject]; [TypeController]; [TypeMessage]; [TypeString]; [TypeItem])]
impl name {
    pub fn get_value(&self) -> u8 {
        self.value
    }
}

impl TypeWord {
    pub fn get_value(&self) -> u16 {
        self.value
    }
}

impl TypeGoto {
    pub fn get_value(&self) -> i16 {
        self.value
    }
}

pub const fn type_var_from_u8(n:u8) -> TypeVar {
    TypeVar {value:n }
}

pub const fn type_object_from_u8(n:u8) -> TypeObject {
    TypeObject {value:n }
}

pub const fn type_flag_from_u8(n:u8) -> TypeFlag {
    TypeFlag {value:n }
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

impl Into<usize> for TypeGoto {
    fn into(self) -> usize {
        self.value as usize
    }
}

impl ops::Add<TypeGoto> for TypeGoto {
    type Output = TypeGoto;
    fn add(self, rhs:TypeGoto) -> Self::Output {
        let a:i16 = self.value;
        let b:i16 = rhs.value;
        (a+b).into()
    }
}

#[derive(IntoStaticStr,Debug)]
pub enum ConditionOperation {
    EqualN((TypeVar,TypeNum)),
    EqualV((TypeVar,TypeVar)),
    LessN((TypeVar,TypeNum)),
    LessV((TypeVar,TypeVar)),
    GreaterN((TypeVar,TypeNum)),
    GreaterV((TypeVar,TypeVar)),
    IsSet((TypeFlag,)),
    IsSetV((TypeVar,)),
    Has((TypeItem,)),
    ObjInRoom((TypeItem,TypeVar)),
    PosN((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
    Controller((TypeController,)),
    HaveKey(()),
    Said((Vec<TypeWord>,)),
    CompareStrings((TypeString,TypeString)),
    ObjInBox((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
    CenterPosN((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
    RightPosN((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
}

#[derive(IntoStaticStr,Debug)]
pub enum LogicChange {
    #[strum(serialize = "")]
    Normal((ConditionOperation,)),
    Not((ConditionOperation,)),
    Or((Vec<LogicChange>,)),
}

#[derive(IntoStaticStr,Debug)]
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
    LoadLogicV((TypeVar,)),
    Call((TypeNum,)),
    CallV((TypeVar,)),
    LoadPic((TypeVar,)),
    DrawPic((TypeVar,)),
    ShowPic(()),
    OverlayPic((TypeVar,)),
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
    ObjectOnAnything((TypeObject,)),
    IgnoreObjs((TypeObject,)),
    ObserveObjs((TypeObject,)),
    Distance((TypeObject,TypeObject,TypeVar)),
    StopCycling((TypeObject,)),
    StartCycling((TypeObject,)),
    NormalCycle((TypeObject,)),
    EndOfLoop((TypeObject,TypeFlag)),
    ReverseLoop((TypeObject,TypeFlag)),
    ReverseCycle((TypeObject,)),
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
    GetDir((TypeObject,TypeVar)),
    IgnoreBlocks((TypeObject,)),
    ObserveBlocks((TypeObject,)),
    Block((TypeNum,TypeNum,TypeNum,TypeNum)),
    Unblock(()),
    Get((TypeItem,)),
    GetV((TypeVar,)),
    Drop((TypeItem,)),
    Put((TypeItem,TypeNum)),
    PutV((TypeVar,TypeVar)),
    GetRoomV((TypeVar,TypeVar)),
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
    ScriptSize((TypeNum,)),
    Version(()),
    SetGameID((TypeMessage,)),
    Log((TypeMessage,)),
    SetScanStart(()),
    ResetScanStart(()),
    RepositionTo((TypeObject,TypeNum,TypeNum)),
    RepositionToV((TypeObject,TypeVar,TypeVar)),
    TraceInfo((TypeNum,TypeNum,TypeNum)),
    PrintAtV0((TypeMessage,TypeNum,TypeNum)),
    PrintAtV1((TypeMessage,TypeNum,TypeNum,TypeNum)),
    PrintAtVV0((TypeVar,TypeNum,TypeNum)),
    PrintAtVV1((TypeVar,TypeNum,TypeNum,TypeNum)),
    ClearTextRect((TypeNum,TypeNum,TypeNum,TypeNum,TypeNum)),
    SetMenu((TypeMessage,)),
    SetMenuMember((TypeMessage,TypeController)),
    SubmitMenu(()),
    DisableMember((TypeController,)),
    EnableMember((TypeController,)),
    MenuInput(()),
    ShowObjV((TypeVar,)),
    OpenDialog(()),
    CloseDialog(()),
    CloseWindow(()),
    MulN((TypeVar,TypeNum)),
    MulV((TypeVar,TypeVar)),
    DivN((TypeVar,TypeNum)),
    DivV((TypeVar,TypeVar)),
    Goto((TypeGoto,)),
    If((Vec<LogicChange>,TypeGoto)),
}

impl LogicMessages {
    fn new(text_slice: &[u8],compression:ResourceCompression) -> Result<LogicMessages,&'static str> {

        // unpack the text data first
        let mut strings:Vec<String> = vec!["".to_string()]; // Push [0] "" string, since messages start counting from 1

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
        let decrypt = match compression {
            ResourceCompression::None => "Avis Durgan",     // TODO Alex Simkin detection
            ResourceCompression::LZW => "\0",
            ResourceCompression::Picture => panic!("This should never occur"),
        };
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

    fn make_empty() -> LogicMessages {
        LogicMessages { strings: Vec::new() }
    }

}

impl LogicResource {
    pub fn new(volume:&Volume, entry: &ResourceDirectoryEntry, version:&ResourcesVersion) -> Result<LogicResource, &'static str> {

        let mut t=VolumeCache::new();
        let data_slice = volume.fetch_data_slice(&mut t,entry).expect("Expected to be able to fetch slice from entry");
        let slice = data_slice.0;

        if slice.len() < 2 {
            let logic_messages = LogicMessages::make_empty();
            let logic_sequence = LogicSequence::make_empty();
            return Ok(LogicResource { logic_messages, logic_sequence });
        }

        let mut slice_iter = slice.iter();

        let lsb_pos = slice_iter.next().unwrap();
        let msb_pos = slice_iter.next().unwrap();
        let position:usize = *msb_pos as usize;
        let position = position<<8;
        let position = position + (*lsb_pos as usize);
        let text_start = position;

        let logic_slice = &slice[2..text_start+2];
        let text_slice = &slice[text_start+2..];

        let logic_messages = LogicMessages::new(text_slice,data_slice.1).expect("Error : ");
        let logic_sequence = LogicSequence::new(logic_slice,version).expect("Error : ");

        Ok(LogicResource {logic_sequence, logic_messages})
    }


    pub fn get_logic_sequence(&self) -> &LogicSequence {
        &self.logic_sequence
    }

    pub fn get_logic_messages(&self) -> &LogicMessages {
        &&self.logic_messages
    }

    fn disassemble_words(words:&Words,word_num:u16) -> String {
        let mut string = format!("word:{}",word_num);
        if word_num == 1 {
            string+="<any>";
        } else if word_num == 9999 {
            string+="<rest of line>";
        } else {
            let match_words = words.fetch_all(word_num);
            string+="(";
            for (index,word) in match_words.into_iter().enumerate() {
                if index !=0 {
                    string+=" || ";
                }
                string+=format!("\"{}\"",word).as_str();
            }
            string+=")";
        }

        string
    }

    fn param_dis_num(t:&TypeNum) -> String {
        format!("{}",t.value)
    }

    fn param_dis_flag(t:&TypeFlag) -> String {
        format!("flag:{}",t.value)
    }

    fn param_dis_var(t:&TypeVar) -> String {
        format!("var:{}",t.value)
    }

    fn param_dis_object(t:&TypeObject) -> String {
        format!("obj:{}",t.value)
    }

    fn param_dis_item(t:&TypeItem,items:&Objects) -> String {
        format!("item:{}\"{}\"",t.value, items.objects[t.value as usize].name)
    }

    fn param_dis_controller(t:&TypeController) -> String {
        format!("ctr:{}",t.value)
    }

    fn param_dis_message(&self, t:&TypeMessage) -> String {
        format!("msg:{}\"{}\"",t.value,self.logic_messages.strings[(t.value) as usize])
    }

    fn param_dis_string(t:&TypeString) -> String {
        format!("str:{}",t.value)
    }

    fn param_dis_word(t:&TypeWord,words:&Words) -> String {
        Self::disassemble_words(words,t.value)
    }

    fn param_dis_said(t:&[TypeWord],words:&Words) -> String {
        let mut string = String::new();
        for (index,w) in t.iter().enumerate() {
            if index != 0 {
                string+=",";
            }
            string+=Self::param_dis_word(w,words).as_str();
        }
        string
    }

    pub fn logic_args_disassemble(operation:&ConditionOperation,words:&Words,items:&Objects) -> String {
        return match operation {
            ConditionOperation::RightPosN(a) |
            ConditionOperation::CenterPosN(a) |
            ConditionOperation::PosN(a) |
            ConditionOperation::ObjInBox(a) => format!("{},{},{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3),Self::param_dis_num(&a.4)),
            ConditionOperation::CompareStrings(a) => format!("{},{}",Self::param_dis_string(&a.0),Self::param_dis_string(&a.1)),
            ConditionOperation::Said(a) => Self::param_dis_said(&a.0, words),
            ConditionOperation::HaveKey(_) => String::from(""),
            ConditionOperation::Controller(a) => Self::param_dis_controller(&a.0),
            ConditionOperation::ObjInRoom(a) => format!("{} {}",Self::param_dis_item(&a.0, items),Self::param_dis_var(&a.1)),
            ConditionOperation::Has(a) => Self::param_dis_item(&a.0, items),
            ConditionOperation::IsSetV(a) => Self::param_dis_var(&a.0),
            ConditionOperation::IsSet(a) => Self::param_dis_flag(&a.0),
            ConditionOperation::GreaterV(a) |
            ConditionOperation::LessV(a) |
            ConditionOperation::EqualV(a) => return format!("{},{}",Self::param_dis_var(&a.0),Self::param_dis_var(&a.1)),
            ConditionOperation::GreaterN(a) |
            ConditionOperation::LessN(a) |
            ConditionOperation::EqualN(a) => return format!("{},{}",Self::param_dis_var(&a.0),Self::param_dis_num(&a.1)),
        }
    }

    pub fn logic_operation_disassemble(operation:&ConditionOperation,words:&Words,items:&Objects) -> String {
        let string = Self::logic_args_disassemble(operation,words,items);
        String::new() + operation.into() + "(" + &string + ")"
    }

    pub fn logic_disassemble(logic:&[LogicChange],is_or:bool,words:&Words,items:&Objects) -> String {
        let mut string = String::new();
        for (index,l) in logic.iter().enumerate() {
            if index!=0 {
                if is_or {
                    string += " || ";
                } else {
                    string += " && ";
                }
            }
            string += &match l {
                LogicChange::Normal((e,)) => Self::logic_operation_disassemble(e,words,items),
                LogicChange::Not((e,)) => String::from("!")+Self::logic_operation_disassemble(e,words,items).as_str(),
                LogicChange::Or((e,)) => String::from("( ")+Self::logic_disassemble(e,true,words,items).as_str()+" )",
            };
        }
        string
    }

    pub fn action_args_disassemble(&self,action:&ActionOperation,_words:&Words,items:&Objects) -> String {
        return match action {
            ActionOperation::Return(_) |
            ActionOperation::ShowPic(_) |
            ActionOperation::UnanimateAll(_) |
            ActionOperation::Unblock(_) |
            ActionOperation::StopSound(_) |
            ActionOperation::TextScreen(_) |
            ActionOperation::Graphics(_) |
            ActionOperation::StatusLineOn(_) |
            ActionOperation::StatusLineOff(_) |
            ActionOperation::PreventInput(_) |
            ActionOperation::AcceptInput(_) |
            ActionOperation::Status(_) |
            ActionOperation::SaveGame(_) |
            ActionOperation::RestoreGame(_) |
            ActionOperation::RestartGame(_) |
            ActionOperation::ProgramControl(_) |
            ActionOperation::PlayerControl(_) |
            ActionOperation::QuitV0(_) |
            ActionOperation::ShowMem(_) |
            ActionOperation::Pause(_) |
            ActionOperation::EchoLine(_) |
            ActionOperation::CancelLine(_) |
            ActionOperation::InitJoy(_) |
            ActionOperation::ToggleMonitor(_) |
            ActionOperation::ShowPriScreen(_) |
            ActionOperation::SubmitMenu(_) |
            ActionOperation::MenuInput(_) |
            ActionOperation::SetScanStart(_) |
            ActionOperation::ResetScanStart(_) |
            ActionOperation::CloseWindow(_) |
            ActionOperation::OpenDialog(_) |
            ActionOperation::CloseDialog(_) |
            ActionOperation::Version(_) => String::new(),
            ActionOperation::Increment(a) |
            ActionOperation::Decrement(a) |
            ActionOperation::SetV(a) |
            ActionOperation::ResetV(a) |
            ActionOperation::NewRoomV(a) |
            ActionOperation::CallV(a) |
            ActionOperation::LoadPic(a) |
            ActionOperation::DrawPic(a) |
            ActionOperation::DiscardPic(a) |
            ActionOperation::OverlayPic(a) |
            ActionOperation::LoadViewV(a) |
            ActionOperation::GetV(a) |
            ActionOperation::PrintV(a) |
            ActionOperation::ShowObjV(a) |
            ActionOperation::LoadLogicV(a) |
            ActionOperation::ObjStatusV(a) => Self::param_dis_var(&a.0),
            ActionOperation::NewRoom(a) |
            ActionOperation::LoadLogic(a) |
            ActionOperation::Call(a) |
            ActionOperation::LoadView(a) |
            ActionOperation::DiscardView(a) |
            ActionOperation::SetHorizon(a) |
            ActionOperation::LoadSound(a) |
            ActionOperation::ShakeScreen(a) |
            ActionOperation::ShowObj(a) |
            ActionOperation::ScriptSize(a) |
            ActionOperation::QuitV1(a) => Self::param_dis_num(&a.0),
            ActionOperation::Set(a) |
            ActionOperation::Reset(a) |
            ActionOperation::Toggle(a) => Self::param_dis_flag(&a.0),
            ActionOperation::Draw(a) |
            ActionOperation::Erase(a) |
            ActionOperation::FixLoop(a) |
            ActionOperation::ReleaseLoop(a) |
            ActionOperation::ReleasePriority(a)|
            ActionOperation::StopUpdate(a) |
            ActionOperation::StartUpdate(a) |
            ActionOperation::ForceUpdate(a) |
            ActionOperation::IgnoreHorizon(a) |
            ActionOperation::ObserveHorizon(a) |
            ActionOperation::ObjectOnWater(a) |
            ActionOperation::ObjectOnLand(a) |
            ActionOperation::ObjectOnAnything(a) |
            ActionOperation::IgnoreObjs(a) |
            ActionOperation::ObserveObjs(a) |
            ActionOperation::StopCycling(a) |
            ActionOperation::StartCycling(a) |
            ActionOperation::NormalCycle(a) |
            ActionOperation::StopMotion(a) |
            ActionOperation::StartMotion(a) |
            ActionOperation::Wander(a) |
            ActionOperation::NormalMotion(a) |
            ActionOperation::IgnoreBlocks(a) |
            ActionOperation::AnimateObj(a) |
            ActionOperation::ReverseCycle(a) |
            ActionOperation::ObserveBlocks(a) => Self::param_dis_object(&a.0),
            ActionOperation::Get(a) |
            ActionOperation::Drop(a) => Self::param_dis_item(&a.0, items),
            ActionOperation::Print(a) |
            ActionOperation::SetMenu(a) |
            ActionOperation::SetCursorChar(a) |
            ActionOperation::Log(a) |
            ActionOperation::SetGameID(a) => self.param_dis_message(&a.0),
            ActionOperation::Parse(a) => Self::param_dis_string(&a.0),
            ActionOperation::EnableMember(a) |
            ActionOperation::DisableMember(a) => Self::param_dis_controller(&a.0),
            ActionOperation::SetTextAttribute(a) => format!("{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1)),
            ActionOperation::Sound(a) => format!("{},{}",Self::param_dis_num(&a.0),Self::param_dis_flag(&a.1)),
            ActionOperation::AddN(a) |
            ActionOperation::SubN(a) |
            ActionOperation::LIndirectN(a) |
            ActionOperation::MulN(a) |
            ActionOperation::DivN(a) |
            ActionOperation::AssignN(a) => format!("{},{}",Self::param_dis_var(&a.0),Self::param_dis_num(&a.1)),
            ActionOperation::AddV(a) |
            ActionOperation::SubV(a) |
            ActionOperation::GetRoomV(a) |
            ActionOperation::PutV(a) |
            ActionOperation::LIndirectV(a) |
            ActionOperation::RIndirect(a) |
            ActionOperation::MulV(a) |
            ActionOperation::DivV(a) |
            ActionOperation::AssignV(a) => format!("{},{}",Self::param_dis_var(&a.0),Self::param_dis_var(&a.1)),
            ActionOperation::Put(a) => format!("{},{}",Self::param_dis_item(&a.0,items),Self::param_dis_num(&a.1)),
            ActionOperation::SetView(a) |
            ActionOperation::SetLoop(a) |
            ActionOperation::SetCel(a) |
            ActionOperation::SetPriority(a) => format!("{},{}",Self::param_dis_object(&a.0),Self::param_dis_num(&a.1)),
            ActionOperation::SetViewV(a) |
            ActionOperation::SetLoopV(a) |
            ActionOperation::SetCelV(a) |
            ActionOperation::LastCel(a) |
            ActionOperation::CurrentCel(a) |
            ActionOperation::CurrentLoop(a) |
            ActionOperation::CurrentView(a) |
            ActionOperation::SetPriorityV(a) |
            ActionOperation::GetPriority(a) |
            ActionOperation::CycleTime(a) |
            ActionOperation::StepSize(a) |
            ActionOperation::StepTime(a) |
            ActionOperation::GetDir(a) |
            ActionOperation::SetDir(a) => format!("{},{}",Self::param_dis_object(&a.0),Self::param_dis_var(&a.1)),
            ActionOperation::EndOfLoop(a) |
            ActionOperation::ReverseLoop(a) => format!("{},{}",Self::param_dis_object(&a.0),Self::param_dis_flag(&a.1)),
            ActionOperation::SetString(a) => format!("{},{}",Self::param_dis_string(&a.0),self.param_dis_message(&a.1)),
            ActionOperation::GetNum(a) => format!("{},{}",self.param_dis_message(&a.0),Self::param_dis_var(&a.1)),
            ActionOperation::SetMenuMember(a) => format!("{},{}",self.param_dis_message(&a.0),Self::param_dis_controller(&a.1)),
            ActionOperation::ClearLines(a) |
            ActionOperation::TraceInfo(a) |
            ActionOperation::ConfigureScreen(a) => format!("{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2)),
            ActionOperation::Random(a) => format!("{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_var(&a.2)),
            ActionOperation::Display(a) => format!("{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),self.param_dis_message(&a.2)),
            ActionOperation::SetKey(a) => format!("{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_controller(&a.2)),
            ActionOperation::DisplayV(a) => format!("{},{},{}",Self::param_dis_var(&a.0),Self::param_dis_var(&a.1),Self::param_dis_var(&a.2)),
            ActionOperation::RepositionTo(a) |
            ActionOperation::Position(a) => format!("{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2)),
            ActionOperation::PositionV(a) |
            ActionOperation::GetPosN(a) |
            ActionOperation::RepositionToV(a) |
            ActionOperation::Reposition(a) => format!("{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_var(&a.1),Self::param_dis_var(&a.2)),
            ActionOperation::Distance(a) => format!("{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_object(&a.1),Self::param_dis_var(&a.2)),
            ActionOperation::FollowEgo(a) => format!("{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_num(&a.1),Self::param_dis_flag(&a.2)),
            ActionOperation::PrintAtVV0(a) => format!("{},{},{}",Self::param_dis_var(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2)),
            ActionOperation::PrintAtV0(a) => format!("{},{},{}",self.param_dis_message(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2)),
            ActionOperation::Block(a) => format!("{},{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3)),
            ActionOperation::PrintAtVV1(a) => format!("{},{},{},{}",Self::param_dis_var(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3)),
            ActionOperation::PrintAtV1(a) => format!("{},{},{},{}",self.param_dis_message(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3)),
            ActionOperation::ClearTextRect(a) => format!("{},{},{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3),Self::param_dis_num(&a.4)),
            ActionOperation::MoveObj(a) => format!("{},{},{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3),Self::param_dis_flag(&a.4)),
            ActionOperation::MoveObjV(a) => format!("{},{},{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_var(&a.1),Self::param_dis_var(&a.2),Self::param_dis_var(&a.3),Self::param_dis_flag(&a.4)),
            ActionOperation::GetString(a) => format!("{},{},{},{},{}",Self::param_dis_string(&a.0),self.param_dis_message(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3),Self::param_dis_num(&a.4)),
            ActionOperation::AddToPic(a) => format!("{},{},{},{},{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3),Self::param_dis_num(&a.4),Self::param_dis_num(&a.5),Self::param_dis_num(&a.6)),
            ActionOperation::AddToPicV(a) => format!("{},{},{},{},{},{},{}",Self::param_dis_var(&a.0),Self::param_dis_var(&a.1),Self::param_dis_var(&a.2),Self::param_dis_var(&a.3),Self::param_dis_var(&a.4),Self::param_dis_var(&a.5),Self::param_dis_var(&a.6)),
            ActionOperation::Goto(_) => panic!("Should not be reached"),
            ActionOperation::If(_) => panic!("Should not be reached"),
        }
    }

    pub fn instruction_disassemble(&self,action:&ActionOperation,words:&Words,items:&Objects) -> String {

        let s:&'static str = action.into();
        return match action {
            ActionOperation::If((logic,_)) => format!("{} ( {} )",s, Self::logic_disassemble(logic,false,words,items)),
            ActionOperation::Goto(a) => format!("{} label_{}",s, self.logic_sequence.labels[&a.0].operation_offset),
            _ => format!("{}({})",s,self.action_args_disassemble(action,words,items)),
        };
    }

    pub fn disassemble(&self,items:&Objects,words:&Words) {

        for g in &self.logic_sequence.labels {
            println!("{:?}",g.0);
        }

        for (_,s) in self.get_disassembly_iterator(words, items) {
            println!("{s}");
        }
    }

    pub fn get_disassembly_iterator<'a>(&'a self,words:&'a Words,items:&'a Objects) -> LogicResourceDisassemblyIterator {
        LogicResourceDisassemblyIterator { logic_resource: &self, words, items, indent: 2, offs: 0, temp_string_vec:VecDeque::new() }
    }
}

pub struct LogicResourceDisassemblyIterator<'a> {
    logic_resource:&'a LogicResource,
    words:&'a Words,
    items:&'a Objects,
    indent:usize,
    offs:usize,
    temp_string_vec:VecDeque<(Option<usize>,String)>,
}

impl<'a> Iterator for LogicResourceDisassemblyIterator<'a> {
    type Item = (Option<usize>,String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.temp_string_vec.is_empty() {
            if self.offs >= self.logic_resource.get_logic_sequence().operations.len() {
                return None;
            }
            let logic_operation = &self.logic_resource.get_logic_sequence().operations[self.offs];
            if let Some(label) = self.logic_resource.get_logic_sequence().labels.get(&logic_operation.address) {
                for _ in 0..label.if_destination_cnt {
                    self.indent-=2;
                    self.temp_string_vec.push_back((None,format!("{:indent$}}}","",indent=self.indent)));
                }
                if label.is_goto_destination {
                    self.temp_string_vec.push_back((None,format!("label_{}:",label.operation_offset)));
                } 
            }

            self.temp_string_vec.push_back((Some(self.offs),format!("{:indent$}{v}","",v=self.logic_resource.instruction_disassemble(&logic_operation.action,self.words,self.items),indent=self.indent)));

            if let ActionOperation::If(_) = logic_operation.action { self.temp_string_vec.push_back((None,format!("{:indent$}{{","",indent=self.indent))); self.indent+=2; }

            self.offs+=1;
        }
        //Otherwise a string should have been pushed into the vec by above (unless we hit the none return)
        Some(self.temp_string_vec.pop_front().unwrap())
    }
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
        Ok((msb<<8)+lsb)
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
        Ok((msb<<8)+lsb)
    }

    fn parse_goto(iter:&mut std::slice::Iter<u8>) -> Result<TypeGoto, &'static str> {
        Ok(Self::read_little_endian_i16(iter)?.into())
    }

    fn parse_message(iter:&mut std::slice::Iter<u8>) -> Result<TypeMessage, &'static str> {
        let m = iter.next();
        if m.is_none() {
            return Err("Expected Message, but reached end of iterator");
        }
        Ok((*m.unwrap()).into())
    }

    fn parse_string(iter:&mut std::slice::Iter<u8>) -> Result<TypeString, &'static str> {
        let s = iter.next();
        if s.is_none() {
            return Err("Expected String, but reached end of iterator");
        }
        Ok((*s.unwrap()).into())
    }

    fn parse_object(iter:&mut std::slice::Iter<u8>) -> Result<TypeObject, &'static str> {
        let o = iter.next();
        if o.is_none() {
            return Err("Expected Object, but reached end of iterator");
        }
        Ok((*o.unwrap()).into())
    }

    fn parse_controller(iter:&mut std::slice::Iter<u8>) -> Result<TypeController, &'static str> {
        let c = iter.next();
        if c.is_none() {
            return Err("Expected Controller, but reached end of iterator");
        }
        Ok((*c.unwrap()).into())
    }

    fn parse_item(iter:&mut std::slice::Iter<u8>) -> Result<TypeItem, &'static str> {
        let i = iter.next();
        if i.is_none() {
            return Err("Expected Item, but reached end of iterator");
        }
        Ok((*i.unwrap()).into())
    }

    fn parse_flag(iter:&mut std::slice::Iter<u8>) -> Result<TypeFlag, &'static str> {
        let f = iter.next();
        if f.is_none() {
            return Err("Expected TypeFlag, but reached end of iterator");
        }
        Ok((*f.unwrap()).into())
    }

    fn parse_var(iter:&mut std::slice::Iter<u8>) -> Result<TypeVar, &'static str> {
        let v = iter.next();
        if v.is_none() {
            return Err("Expected TypeVariable, but reached end of iterator");
        }
        Ok((*v.unwrap()).into())
    }
    
    fn parse_num(iter:&mut std::slice::Iter<u8>) -> Result<TypeNum, &'static str> {
        let n = iter.next();
        if n.is_none() {
            return Err("Expected TypeNumber, but reached end of iterator");
        }
        Ok((*n.unwrap()).into())
    }

    fn parse_word(iter:&mut std::slice::Iter<u8>) -> Result<TypeWord, &'static str> {
        Ok(Self::read_little_endian_u16(iter)?.into())
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
        Ok(words)
    }

    fn parse_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_num_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeFlag), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_flag(iter)?))
    }
    
    fn parse_var_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeNum), &'static str> {
        Ok((Self::parse_var(iter)?,Self::parse_num(iter)?))
    }
    
    fn parse_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeVar), &'static str> {
        Ok((Self::parse_var(iter)?,Self::parse_var(iter)?))
    }
    
    fn parse_item_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeItem,TypeNum), &'static str> {
        Ok((Self::parse_item(iter)?,Self::parse_num(iter)?))
    }
    
    fn parse_item_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeItem,TypeVar), &'static str> {
        Ok((Self::parse_item(iter)?,Self::parse_var(iter)?))
    }
    
    fn parse_object_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_num(iter)?))
    }
    
    fn parse_object_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeVar), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_var(iter)?))
    }

    fn parse_object_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeFlag), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_flag(iter)?))
    }

    fn parse_string_string(iter:&mut std::slice::Iter<u8>) -> Result<(TypeString,TypeString), &'static str> {
        Ok((Self::parse_string(iter)?,Self::parse_string(iter)?))
    }

    fn parse_string_message(iter:&mut std::slice::Iter<u8>) -> Result<(TypeString,TypeMessage), &'static str> {
        Ok((Self::parse_string(iter)?,Self::parse_message(iter)?))
    }

    fn parse_message_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeMessage,TypeVar), &'static str> {
        Ok((Self::parse_message(iter)?,Self::parse_var(iter)?))
    }
    
    fn parse_message_controller(iter:&mut std::slice::Iter<u8>) -> Result<(TypeMessage,TypeController), &'static str> {
        Ok((Self::parse_message(iter)?,Self::parse_controller(iter)?))
    }

    fn parse_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_num_num_message(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeMessage), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_message(iter)?))
    }

    fn parse_num_num_controller(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeController), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_controller(iter)?))
    }

    fn parse_num_num_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeVar), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_var(iter)?))
    }

    fn parse_var_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeVar,TypeVar), &'static str> {
        Ok((Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?))
    }

    fn parse_object_object_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeObject,TypeVar), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_object(iter)?,Self::parse_var(iter)?))
    }

    fn parse_var_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_var(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_message_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeMessage,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_message(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_object_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeVar,TypeVar), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?))
    }

    fn parse_object_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_object_num_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeFlag), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_flag(iter)?))
    }

    fn parse_var_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_var(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_message_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeMessage,TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_message(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_num_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }
    
    fn parse_num_num_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }
    
    fn parse_object_num_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }
    
    fn parse_object_num_num_num_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeNum,TypeNum,TypeNum,TypeFlag), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_flag(iter)?))
    }
    
    fn parse_object_var_var_var_flag(iter:&mut std::slice::Iter<u8>) -> Result<(TypeObject,TypeVar,TypeVar,TypeVar,TypeFlag), &'static str> {
        Ok((Self::parse_object(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_flag(iter)?))
    }
    
    fn parse_string_message_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeString,TypeMessage,TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_string(iter)?,Self::parse_message(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }

    fn parse_num_num_num_num_num_num_num(iter:&mut std::slice::Iter<u8>) -> Result<(TypeNum,TypeNum,TypeNum,TypeNum,TypeNum,TypeNum,TypeNum), &'static str> {
        Ok((Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?,Self::parse_num(iter)?))
    }
    
    fn parse_var_var_var_var_var_var_var(iter:&mut std::slice::Iter<u8>) -> Result<(TypeVar,TypeVar,TypeVar,TypeVar,TypeVar,TypeVar,TypeVar), &'static str> {
        Ok((Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?,Self::parse_var(iter)?))
    }
    
    fn parse_condition_with_code(iter:&mut std::slice::Iter<u8>, code:u8) -> Result<ConditionOperation, &'static str> {
        match code {
            0x12 => Ok(ConditionOperation::RightPosN(Self::parse_object_num_num_num_num(iter)?)),
            0x11 => Ok(ConditionOperation::CenterPosN(Self::parse_object_num_num_num_num(iter)?)),
            0x10 => Ok(ConditionOperation::ObjInBox(Self::parse_object_num_num_num_num(iter)?)),
            0x0F => Ok(ConditionOperation::CompareStrings(Self::parse_string_string(iter)?)),
            0x0E => Ok(ConditionOperation::Said((Self::parse_said(iter)?,))),
            0x0D => Ok(ConditionOperation::HaveKey(())),
            0x0C => Ok(ConditionOperation::Controller((Self::parse_controller(iter)?,))),
            0x0B => Ok(ConditionOperation::PosN(Self::parse_object_num_num_num_num(iter)?)),
            0x0A => Ok(ConditionOperation::ObjInRoom(Self::parse_item_var(iter)?)),
            0x09 => Ok(ConditionOperation::Has((Self::parse_item(iter)?,))),
            0x08 => Ok(ConditionOperation::IsSetV((Self::parse_var(iter)?,))),
            0x07 => Ok(ConditionOperation::IsSet((Self::parse_flag(iter)?,))),
            0x06 => Ok(ConditionOperation::GreaterV(Self::parse_var_var(iter)?)),
            0x05 => Ok(ConditionOperation::GreaterN(Self::parse_var_num(iter)?)),
            0x04 => Ok(ConditionOperation::LessV(Self::parse_var_var(iter)?)),
            0x03 => Ok(ConditionOperation::LessN(Self::parse_var_num(iter)?)),
            0x02 => Ok(ConditionOperation::EqualV(Self::parse_var_var(iter)?)),
            0x01 => Ok(ConditionOperation::EqualN(Self::parse_var_num(iter)?)),
            _ => Err("Unexpected ConditionOperation {code:02X}"),
        }
    }

    fn parse_condition(iter:&mut std::slice::Iter<u8>) -> Result<ConditionOperation, &'static str> {
        let code = iter.next();
        if code.is_none() {
            return Err("Expected condition code, but reached end of iterator");
        }
        Self::parse_condition_with_code(iter,*(code.unwrap()))
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
        Ok(or)
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

        Ok((conditions,pos))
    }

    fn new(logic_slice: &[u8],version:&ResourcesVersion) -> Result<LogicSequence,&'static str> {

        let mut iter = logic_slice.iter();

        let mut operations:Vec<LogicOperation> = Vec::new();
        let mut offsets:HashMap<TypeGoto, usize>=HashMap::new();
        let mut offsets_rev:HashMap<usize, TypeGoto>=HashMap::new();
        let initial_size = logic_slice.len();

        let version_2089 = &ResourcesVersion::new("2.089");
        let version_2400 = &ResourcesVersion::new("2.400");

        operations.reserve(initial_size);  // over allocate then shrink to fit at end of process (over allocates,because there are operands mixed into the stream)
        offsets.reserve(initial_size);
        offsets_rev.reserve(initial_size);

        while let Some(b) = iter.next()
        {
            let program_position = initial_size - iter.as_slice().len() -1;
            let address:TypeGoto = (program_position as i16).into();
            offsets.insert(address, operations.len());
            offsets_rev.insert(operations.len(),address);
            let action = match b {
                0xFF => ActionOperation::If(Self::parse_vlogic_change_goto(&mut iter)?),
                0xFE => ActionOperation::Goto((Self::parse_goto(&mut iter)?,)),
                0xA9 => ActionOperation::CloseWindow(()),
                0xA8 => ActionOperation::DivV(Self::parse_var_var(&mut iter)?),
                0xA7 => ActionOperation::DivN(Self::parse_var_num(&mut iter)?),
                0xA6 => ActionOperation::MulV(Self::parse_var_var(&mut iter)?),
                0xA5 => ActionOperation::MulN(Self::parse_var_num(&mut iter)?),
                0xA4 => ActionOperation::CloseDialog(()),
                0xA3 => ActionOperation::OpenDialog(()),
                0xA2 => ActionOperation::ShowObjV((Self::parse_var(&mut iter)?,)),
                0xA1 => ActionOperation::MenuInput(()),
                0xA0 => ActionOperation::DisableMember((Self::parse_controller(&mut iter)?,)),
                0x9F => ActionOperation::EnableMember((Self::parse_controller(&mut iter)?,)),
                0x9E => ActionOperation::SubmitMenu(()),
                0x9D => ActionOperation::SetMenuMember(Self::parse_message_controller(&mut iter)?),
                0x9C => ActionOperation::SetMenu((Self::parse_message(&mut iter)?,)),
                0x9A => ActionOperation::ClearTextRect(Self::parse_num_num_num_num_num(&mut iter)?),
                0x98 => if version >= version_2089 && version <= version_2400 {ActionOperation::PrintAtVV0(Self::parse_var_num_num(&mut iter)?) } else {ActionOperation::PrintAtVV1(Self::parse_var_num_num_num(&mut iter)?)},
                0x97 => if version >= version_2089 && version <= version_2400 {ActionOperation::PrintAtV0(Self::parse_message_num_num(&mut iter)?) } else {ActionOperation::PrintAtV1(Self::parse_message_num_num_num(&mut iter)?)},
                0x96 => ActionOperation::TraceInfo(Self::parse_num_num_num(&mut iter)?),
                0x94 => ActionOperation::RepositionToV(Self::parse_object_var_var(&mut iter)?),
                0x93 => ActionOperation::RepositionTo(Self::parse_object_num_num(&mut iter)?),
                0x92 => ActionOperation::ResetScanStart(()),
                0x91 => ActionOperation::SetScanStart(()),
                0x90 => ActionOperation::Log((Self::parse_message(&mut iter)?,)),
                0x8F => ActionOperation::SetGameID((Self::parse_message(&mut iter)?,)),
                0x8E => ActionOperation::ScriptSize((Self::parse_num(&mut iter)?,)),
                0x8D => ActionOperation::Version(()),
                0x8C => ActionOperation::ToggleMonitor(()),
                0x8B => ActionOperation::InitJoy(()),
                0x8A => ActionOperation::CancelLine(()),
                0x89 => ActionOperation::EchoLine(()),
                0x88 => ActionOperation::Pause(()),
                0x87 => ActionOperation::ShowMem(()),
                0x86 => if version == version_2089 { ActionOperation::QuitV0(()) } else { ActionOperation::QuitV1((Self::parse_num(&mut iter)?,))},
                0x85 => ActionOperation::ObjStatusV((Self::parse_var(&mut iter)?,)),
                0x84 => ActionOperation::PlayerControl(()),
                0x83 => ActionOperation::ProgramControl(()),
                0x82 => ActionOperation::Random(Self::parse_num_num_var(&mut iter)?),
                0x81 => ActionOperation::ShowObj((Self::parse_num(&mut iter)?,)),
                0x80 => ActionOperation::RestartGame(()),
                0x7E => ActionOperation::RestoreGame(()),
                0x7D => ActionOperation::SaveGame(()),
                0x7C => ActionOperation::Status(()),
                0x7B => ActionOperation::AddToPicV(Self::parse_var_var_var_var_var_var_var(&mut iter)?),
                0x7A => ActionOperation::AddToPic(Self::parse_num_num_num_num_num_num_num(&mut iter)?),
                0x79 => ActionOperation::SetKey(Self::parse_num_num_controller(&mut iter)?),
                0x78 => ActionOperation::AcceptInput(()),
                0x77 => ActionOperation::PreventInput(()),
                0x76 => ActionOperation::GetNum(Self::parse_message_var(&mut iter)?),
                0x75 => ActionOperation::Parse((Self::parse_string(&mut iter)?,)),
                0x73 => ActionOperation::GetString(Self::parse_string_message_num_num_num(&mut iter)?),
                0x72 => ActionOperation::SetString(Self::parse_string_message(&mut iter)?),
                0x71 => ActionOperation::StatusLineOff(()),
                0x70 => ActionOperation::StatusLineOn(()),
                0x6F => ActionOperation::ConfigureScreen(Self::parse_num_num_num(&mut iter)?),
                0x6E => ActionOperation::ShakeScreen((Self::parse_num(&mut iter)?,)),
                0x6D => ActionOperation::SetTextAttribute(Self::parse_num_num(&mut iter)?),
                0x6C => ActionOperation::SetCursorChar((Self::parse_message(&mut iter)?,)),
                0x6B => ActionOperation::Graphics(()),
                0x6A => ActionOperation::TextScreen(()),
                0x69 => ActionOperation::ClearLines(Self::parse_num_num_num(&mut iter)?),
                0x68 => ActionOperation::DisplayV(Self::parse_var_var_var(&mut iter)?),
                0x67 => ActionOperation::Display(Self::parse_num_num_message(&mut iter)?),
                0x66 => ActionOperation::PrintV((Self::parse_var(&mut iter)?,)),
                0x65 => ActionOperation::Print((Self::parse_message(&mut iter)?,)),
                0x64 => ActionOperation::StopSound(()),
                0x63 => ActionOperation::Sound(Self::parse_num_flag(&mut iter)?),
                0x62 => ActionOperation::LoadSound((Self::parse_num(&mut iter)?,)),
                0x61 => ActionOperation::GetRoomV(Self::parse_var_var(&mut iter)?),
                0x60 => ActionOperation::PutV(Self::parse_var_var(&mut iter)?), //Check not item,var
                0x5F => ActionOperation::Put(Self::parse_item_num(&mut iter)?),
                0x5E => ActionOperation::Drop((Self::parse_item(&mut iter)?,)),
                0x5D => ActionOperation::GetV((Self::parse_var(&mut iter)?,)),
                0x5C => ActionOperation::Get((Self::parse_item(&mut iter)?,)),
                0x5B => ActionOperation::Unblock(()),
                0x5A => ActionOperation::Block(Self::parse_num_num_num_num(&mut iter)?),
                0x59 => ActionOperation::ObserveBlocks((Self::parse_object(&mut iter)?,)),
                0x58 => ActionOperation::IgnoreBlocks((Self::parse_object(&mut iter)?,)),
                0x57 => ActionOperation::GetDir(Self::parse_object_var(&mut iter)?),
                0x56 => ActionOperation::SetDir(Self::parse_object_var(&mut iter)?),
                0x55 => ActionOperation::NormalMotion((Self::parse_object(&mut iter)?,)),
                0x54 => ActionOperation::Wander((Self::parse_object(&mut iter)?,)),
                0x53 => ActionOperation::FollowEgo(Self::parse_object_num_flag(&mut iter)?),
                0x52 => ActionOperation::MoveObjV(Self::parse_object_var_var_var_flag(&mut iter)?),
                0x51 => ActionOperation::MoveObj(Self::parse_object_num_num_num_flag(&mut iter)?),
                0x50 => ActionOperation::StepTime(Self::parse_object_var(&mut iter)?),
                0x4F => ActionOperation::StepSize(Self::parse_object_var(&mut iter)?),
                0x4E => ActionOperation::StartMotion((Self::parse_object(&mut iter)?,)),
                0x4D => ActionOperation::StopMotion((Self::parse_object(&mut iter)?,)),
                0x4C => ActionOperation::CycleTime(Self::parse_object_var(&mut iter)?),
                0x4B => ActionOperation::ReverseLoop(Self::parse_object_flag(&mut iter)?),
                0x4A => ActionOperation::ReverseCycle((Self::parse_object(&mut iter)?,)),
                0x49 => ActionOperation::EndOfLoop(Self::parse_object_flag(&mut iter)?),
                0x48 => ActionOperation::NormalCycle((Self::parse_object(&mut iter)?,)),
                0x47 => ActionOperation::StartCycling((Self::parse_object(&mut iter)?,)),
                0x46 => ActionOperation::StopCycling((Self::parse_object(&mut iter)?,)),
                0x45 => ActionOperation::Distance(Self::parse_object_object_var(&mut iter)?),
                0x44 => ActionOperation::ObserveObjs((Self::parse_object(&mut iter)?,)),
                0x43 => ActionOperation::IgnoreObjs((Self::parse_object(&mut iter)?,)),
                0x42 => ActionOperation::ObjectOnAnything((Self::parse_object(&mut iter)?,)),
                0x41 => ActionOperation::ObjectOnLand((Self::parse_object(&mut iter)?,)),
                0x40 => ActionOperation::ObjectOnWater((Self::parse_object(&mut iter)?,)),
                0x3F => ActionOperation::SetHorizon((Self::parse_num(&mut iter)?,)),
                0x3E => ActionOperation::ObserveHorizon((Self::parse_object(&mut iter)?,)),
                0x3D => ActionOperation::IgnoreHorizon((Self::parse_object(&mut iter)?,)),
                0x3C => ActionOperation::ForceUpdate((Self::parse_object(&mut iter)?,)),
                0x3B => ActionOperation::StartUpdate((Self::parse_object(&mut iter)?,)),
                0x3A => ActionOperation::StopUpdate((Self::parse_object(&mut iter)?,)),
                0x39 => ActionOperation::GetPriority(Self::parse_object_var(&mut iter)?),
                0x38 => ActionOperation::ReleasePriority((Self::parse_object(&mut iter)?,)),
                0x37 => ActionOperation::SetPriorityV(Self::parse_object_var(&mut iter)?),
                0x36 => ActionOperation::SetPriority(Self::parse_object_num(&mut iter)?),
                0x34 => ActionOperation::CurrentView(Self::parse_object_var(&mut iter)?),
                0x33 => ActionOperation::CurrentLoop(Self::parse_object_var(&mut iter)?),
                0x32 => ActionOperation::CurrentCel(Self::parse_object_var(&mut iter)?),
                0x31 => ActionOperation::LastCel(Self::parse_object_var(&mut iter)?),
                0x30 => ActionOperation::SetCelV(Self::parse_object_var(&mut iter)?),
                0x2F => ActionOperation::SetCel(Self::parse_object_num(&mut iter)?,),
                0x2E => ActionOperation::ReleaseLoop((Self::parse_object(&mut iter)?,)),
                0x2D => ActionOperation::FixLoop((Self::parse_object(&mut iter)?,)),
                0x2C => ActionOperation::SetLoopV(Self::parse_object_var(&mut iter)?),
                0x2B => ActionOperation::SetLoop(Self::parse_object_num(&mut iter)?),
                0x2A => ActionOperation::SetViewV(Self::parse_object_var(&mut iter)?),
                0x29 => ActionOperation::SetView(Self::parse_object_num(&mut iter)?),
                0x28 => ActionOperation::Reposition(Self::parse_object_var_var(&mut iter)?),
                0x27 => ActionOperation::GetPosN(Self::parse_object_var_var(&mut iter)?),
                0x26 => ActionOperation::PositionV(Self::parse_object_var_var(&mut iter)?),
                0x25 => ActionOperation::Position(Self::parse_object_num_num(&mut iter)?),
                0x24 => ActionOperation::Erase((Self::parse_object(&mut iter)?,)),
                0x23 => ActionOperation::Draw((Self::parse_object(&mut iter)?,)),
                0x22 => ActionOperation::UnanimateAll(()),
                0x21 => ActionOperation::AnimateObj((Self::parse_object(&mut iter)?,)),
                0x20 => ActionOperation::DiscardView((Self::parse_num(&mut iter)?,)),
                0x1F => ActionOperation::LoadViewV((Self::parse_var(&mut iter)?,)),
                0x1E => ActionOperation::LoadView((Self::parse_num(&mut iter)?,)),
                0x1D => ActionOperation::ShowPriScreen(()),
                0x1C => ActionOperation::OverlayPic((Self::parse_var(&mut iter)?,)),
                0x1B => ActionOperation::DiscardPic((Self::parse_var(&mut iter)?,)),
                0x1A => ActionOperation::ShowPic(()),
                0x19 => ActionOperation::DrawPic((Self::parse_var(&mut iter)?,)),
                0x18 => ActionOperation::LoadPic((Self::parse_var(&mut iter)?,)),
                0x17 => ActionOperation::CallV((Self::parse_var(&mut iter)?,)),
                0x16 => ActionOperation::Call((Self::parse_num(&mut iter)?,)),
                0x15 => ActionOperation::LoadLogicV((Self::parse_var(&mut iter)?,)),
                0x14 => ActionOperation::LoadLogic((Self::parse_num(&mut iter)?,)),
                0x13 => ActionOperation::NewRoomV((Self::parse_var(&mut iter)?,)),
                0x12 => ActionOperation::NewRoom((Self::parse_num(&mut iter)?,)),
                0x10 => ActionOperation::ResetV((Self::parse_var(&mut iter)?,)),
                0x0F => ActionOperation::SetV((Self::parse_var(&mut iter)?,)),
                0x0E => ActionOperation::Toggle((Self::parse_flag(&mut iter)?,)),
                0x0D => ActionOperation::Reset((Self::parse_flag(&mut iter)?,)),
                0x0C => ActionOperation::Set((Self::parse_flag(&mut iter)?,)),
                0x0B => ActionOperation::LIndirectN(Self::parse_var_num(&mut iter)?),
                0x0A => ActionOperation::RIndirect(Self::parse_var_var(&mut iter)?),
                0x09 => ActionOperation::LIndirectV(Self::parse_var_var(&mut iter)?),
                0x08 => ActionOperation::SubV(Self::parse_var_var(&mut iter)?),
                0x07 => ActionOperation::SubN(Self::parse_var_num(&mut iter)?),
                0x06 => ActionOperation::AddV(Self::parse_var_var(&mut iter)?),
                0x05 => ActionOperation::AddN(Self::parse_var_num(&mut iter)?),
                0x04 => ActionOperation::AssignV(Self::parse_var_var(&mut iter)?),
                0x03 => ActionOperation::AssignN(Self::parse_var_num(&mut iter)?),
                0x02 => ActionOperation::Decrement((Self::parse_var(&mut iter)?,)),
                0x01 => ActionOperation::Increment((Self::parse_var(&mut iter)?,)),
                0x00 => ActionOperation::Return(()),
                _ => {panic!("Unimplemented action {b:02X}");}
            };

            operations.push(LogicOperation { action,address });
        }

        let mut labels:HashMap<TypeGoto, Label>=HashMap::new();
        labels.reserve(operations.len());
        for (index,op) in operations.iter_mut().enumerate() {
            matches!(op.action, ActionOperation::Goto(_));
            let is_goto= matches!(op.action, ActionOperation::Goto(_));

            let mut destination:TypeGoto = 0i16.into();
            match op.action {
                ActionOperation::Goto((g,)) | ActionOperation::If((_,g)) => {
                    let base_offset=offsets_rev.get(&(index+1)).unwrap();
                    destination = *base_offset+g;

                    if let std::collections::hash_map::Entry::Vacant(e) = labels.entry(destination) {
                        e.insert(Label { is_goto_destination:is_goto, if_destination_cnt: if !is_goto {1} else {0}, operation_offset:offsets[&destination]});
                    } else {
                        if labels[&destination].operation_offset != offsets[&destination] {
                            panic!("WUT!");
                        }
                        if is_goto {
                            labels.get_mut(&destination).unwrap().is_goto_destination=true;
                        } else {
                            labels.get_mut(&destination).unwrap().if_destination_cnt+=1;
                        }
                    }
                },
                _ => {},
            }
            match &mut op.action {
                ActionOperation::Goto((a,)) => *a = destination,
                ActionOperation::If((_,a)) => *a = destination,
                _ => {},
            };
        }

        operations.shrink_to_fit();
        labels.shrink_to_fit();
        Ok(LogicSequence { operations, labels })
    }

    fn make_empty() -> LogicSequence {
        LogicSequence { operations: Vec::new(), labels: HashMap::new() }
    }

    pub fn get_operations(&self) -> &Vec<LogicOperation> {
        &self.operations
    }

    pub fn lookup_offset(&self,goto:&TypeGoto) -> Option<usize> {
        self.labels.get(goto).map(|b| b.operation_offset)
    }

}
