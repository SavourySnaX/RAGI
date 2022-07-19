use std::{collections::HashMap, hash::Hash, ops};

use dir_resource::ResourceDirectoryEntry;
use objects::Objects;
use volume::Volume;
use words::Words;

use strum_macros::IntoStaticStr;

#[derive(Debug,Copy,Clone)] // TODO revisit copy
pub struct Sprite {
    active:bool,
    observing:bool, // treats other objects as obsticles
    cycle:bool,     // cycle loop automatically
    view:u8,
    cloop:u8,
    cel:u8,
    x:u8,           // bottom left corner
    y:u8,
    priority:u8,
}

impl Sprite {
    pub fn new() -> Sprite {
        return Sprite { 
            active: false, 
            observing: false, 
            cycle: false, 
            view: 0, 
            cloop: 0,
            cel: 0,
            x:0, 
            y:0,
            priority:0,
        };
    }

    pub fn get_x(&self) -> u8 {
        return self.x;
    }
    
    pub fn get_y(&self) -> u8 {
        return self.y;
    }

    pub fn set_active(&mut self,b:bool) {
        self.active=b;
    }

    pub fn set_view(&mut self, view:u8) {
        self.view = view;
    }

    pub fn set_observing(&mut self,b:bool) {
        self.observing=b;
    }

    pub fn set_cycling(&mut self,b:bool) {
        self.cycle=b;
    }

    pub fn set_x(&mut self,n:u8) {
        self.x = n;
    }
    
    pub fn set_y(&mut self,n:u8) {
        self.y = n;
    }
    
    pub fn set_priority(&mut self,n:u8) {
        self.priority = n;
    }
   
    pub fn set_loop(&mut self,n:u8) {
        self.cloop = n;
    }
    
    pub fn set_cel(&mut self,n:u8) {
        self.cel = n;
    }

}

#[derive(Debug)]
pub struct LogicState {
    input:bool,
    horizon:u8,
    flag:[bool;256],
    var:[u8;256],
    objects:[Sprite;256],   // overkill, todo add list of active
}

impl LogicState {
    pub fn new() -> LogicState {
        return LogicState {
            input: false,
            horizon: 0,
            flag: [false;256],
            var: [0u8;256],
            objects: [Sprite::new();256],
        }
    }
    pub fn get_flag(&self,f:&TypeFlag) -> bool {
        return self.flag[f.value as usize];
    }

    pub fn get_var(&self,v:&TypeVar) -> u8 {
        return self.var[v.value as usize];
    }

    pub fn get_num(&self,v:&TypeNum) -> u8 {
        return v.value;
    }

    pub fn set_var(&mut self,v:&TypeVar,n:u8) {
        self.var[v.value as usize] = n;
    }

    pub fn set_flag(&mut self,f:&TypeFlag,n:bool) {
        self.flag[f.value as usize] = n;
    }
    
    pub fn set_input(&mut self,b:bool) {
        self.input = b;
    }
    
    pub fn set_horizon(&mut self,h:u8) {
        self.horizon = h;
    }

    pub fn object(&self,o:&TypeObject) -> &Sprite {
        return &self.objects[o.value as usize];
    }

    pub fn mut_object(&mut self,o:&TypeObject) -> &mut Sprite {
        return &mut self.objects[o.value as usize];
    }
}

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
#[derive(Clone,Copy,Debug)]
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
    PosN((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
    Controller((TypeController,)),
    HaveKey(()),
    Said((Vec<TypeWord>,)),
    ObjInBox((TypeObject,TypeNum,TypeNum,TypeNum,TypeNum)),
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
        //let decrypt = "Alex Simkin";
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

    pub fn get_logic_sequence(&self) -> &LogicSequence {
        return &self.logic_sequence;
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

        return string;
    }

    fn param_dis_num(t:&TypeNum) -> String {
        return format!("{}",t.value);
    }

    fn param_dis_flag(t:&TypeFlag) -> String {
        return format!("flag:{}",t.value);
    }

    fn param_dis_var(t:&TypeVar) -> String {
        return format!("var:{}",t.value);
    }

    fn param_dis_object(t:&TypeObject) -> String {
        return format!("obj:{}",t.value);
    }

    fn param_dis_item(t:&TypeItem,items:&Objects) -> String {
        return format!("item:{}\"{}\"",t.value, items.objects[t.value as usize].name);
    }

    fn param_dis_controller(t:&TypeController) -> String {
        return format!("ctr:{}",t.value);
    }

    fn param_dis_message(&self, t:&TypeMessage) -> String {
        return format!("msg:{}\"{}\"",t.value,self.logic_messages.strings[(t.value) as usize]);
    }

    fn param_dis_string(t:&TypeString) -> String {
        return format!("str:{}",t.value);
    }

    fn param_dis_word(t:&TypeWord,words:&Words) -> String {
        return Self::disassemble_words(words,t.value);
    }

    fn param_dis_said(t:&Vec<TypeWord>,words:&Words) -> String {
        let mut string = String::new();
        for (index,w) in t.iter().enumerate() {
            if index != 0 {
                string+=",";
            }
            string+=Self::param_dis_word(w,words).as_str();
        }
        return string;
    }

    pub fn logic_args_disassemble(operation:&ConditionOperation,words:&Words,items:&Objects) -> String {
        return match operation {
            ConditionOperation::RightPosN(a) |
            ConditionOperation::PosN(a) |
            ConditionOperation::ObjInBox(a) => format!("{},{},{},{},{}",Self::param_dis_object(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3),Self::param_dis_num(&a.4)),
            ConditionOperation::Said(a) => Self::param_dis_said(&a.0, words),
            ConditionOperation::HaveKey(_) => String::from(""),
            ConditionOperation::Controller(a) => Self::param_dis_controller(&a.0),
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
        return String::new() + operation.into() + "(" + &string + ")";
    }

    pub fn logic_disassemble(logic:&Vec<LogicChange>,is_or:bool,words:&Words,items:&Objects) -> String {
        let mut string = String::new();
        for (index,l) in logic.iter().enumerate() {
            if index!=0 {
                if is_or {
                    string = string + " || ";
                } else {
                    string = string + " && ";
                }
            }
            string = string + &match l {
                LogicChange::Normal((e,)) => Self::logic_operation_disassemble(e,words,items),
                LogicChange::Not((e,)) => String::from("!")+Self::logic_operation_disassemble(e,words,items).as_str(),
                LogicChange::Or((e,)) => String::from("( ")+Self::logic_disassemble(e,true,words,items).as_str()+" )",
            };
        }
        return string;
    }

    pub fn action_args_disassemble(&self,action:&ActionOperation,words:&Words,items:&Objects) -> String {
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
            ActionOperation::LoadViewV(a) |
            ActionOperation::GetV(a) |
            ActionOperation::PrintV(a) |
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
            ActionOperation::IgnoreObjs(a) |
            ActionOperation::ObserveObjs(a) |
            ActionOperation::StopCycling(a) |
            ActionOperation::StartCycling(a) |
            ActionOperation::StopMotion(a) |
            ActionOperation::StartMotion(a) |
            ActionOperation::Wander(a) |
            ActionOperation::NormalMotion(a) |
            ActionOperation::IgnoreBlocks(a) |
            ActionOperation::AnimateObj(a) |
            ActionOperation::ObserveBlocks(a) => Self::param_dis_object(&a.0),
            ActionOperation::Get(a) |
            ActionOperation::Drop(a) => Self::param_dis_item(&a.0, items),
            ActionOperation::Print(a) |
            ActionOperation::SetCursorChar(a) |
            ActionOperation::SetGameID(a) => self.param_dis_message(&a.0),
            ActionOperation::Parse(a) => Self::param_dis_string(&a.0),
            ActionOperation::SetTextAttribute(a) => format!("{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1)),
            ActionOperation::Sound(a) => format!("{},{}",Self::param_dis_num(&a.0),Self::param_dis_flag(&a.1)),
            ActionOperation::AddN(a) |
            ActionOperation::SubN(a) |
            ActionOperation::LIndirectN(a) |
            ActionOperation::AssignN(a) => format!("{},{}",Self::param_dis_var(&a.0),Self::param_dis_num(&a.1)),
            ActionOperation::AddV(a) |
            ActionOperation::SubV(a) |
            ActionOperation::LIndirectV(a) |
            ActionOperation::RIndirect(a) |
            ActionOperation::AssignV(a) => format!("{},{}",Self::param_dis_var(&a.0),Self::param_dis_var(&a.1)),
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
            ActionOperation::SetDir(a) => format!("{},{}",Self::param_dis_object(&a.0),Self::param_dis_var(&a.1)),
            ActionOperation::EndOfLoop(a) |
            ActionOperation::ReverseLoop(a) => format!("{},{}",Self::param_dis_object(&a.0),Self::param_dis_flag(&a.1)),
            ActionOperation::SetString(a) => format!("{},{}",Self::param_dis_string(&a.0),self.param_dis_message(&a.1)),
            ActionOperation::GetNum(a) => format!("{},{}",self.param_dis_message(&a.0),Self::param_dis_var(&a.1)),
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
            ActionOperation::PrintAtV0(a) => format!("{},{},{}",self.param_dis_message(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2)),
            ActionOperation::Block(a) => format!("{},{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3)),
            ActionOperation::PrintAtV1(a) => format!("{},{},{},{}",self.param_dis_message(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3)),
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

        let mut indent = 2;
        for logic_operation in &self.logic_sequence.operations {
            if let Some(label) = self.logic_sequence.labels.get(&logic_operation.address) {
               for _ in 0..label.if_destination_cnt {
                    indent-=2;
                    println!("{:indent$}}}","",indent=indent);
                }
                if label.is_goto_destination {
                    println!("label_{}:",label.operation_offset);
                } 
            }

            println!("{:indent$}{v}","",v=self.instruction_disassemble(&logic_operation.action,words,items),indent=indent);

            match logic_operation.action {
                ActionOperation::If(_) => { println!("{:indent$}{{","",indent=indent);indent+=2; }
                _ => {}
            }
        }
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
    
    fn parse_condition_with_code(iter:&mut std::slice::Iter<u8>, code:u8) -> Result<ConditionOperation, &'static str> {
        return match code {
            0x12 => Ok(ConditionOperation::RightPosN(Self::parse_object_num_num_num_num(iter)?)),
            0x10 => Ok(ConditionOperation::ObjInBox(Self::parse_object_num_num_num_num(iter)?)),
            0x0E => Ok(ConditionOperation::Said((Self::parse_said(iter)?,))),
            0x0D => Ok(ConditionOperation::HaveKey(())),
            0x0C => Ok(ConditionOperation::Controller((Self::parse_controller(iter)?,))),
            0x0B => Ok(ConditionOperation::PosN(Self::parse_object_num_num_num_num(iter)?)),
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

        let mut operations:Vec<LogicOperation> = Vec::new();
        let mut offsets:HashMap<TypeGoto, usize>=HashMap::new();
        let mut offsets_rev:HashMap<usize, TypeGoto>=HashMap::new();
        let initial_size = logic_slice.len();

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
                0x97 => ActionOperation::PrintAtV0(Self::parse_message_num_num(&mut iter)?),
                0x96 => ActionOperation::TraceInfo(Self::parse_num_num_num(&mut iter)?),
                0x94 => ActionOperation::RepositionToV(Self::parse_object_var_var(&mut iter)?),
                0x93 => ActionOperation::RepositionTo(Self::parse_object_num_num(&mut iter)?),
                0x8F => ActionOperation::SetGameID((Self::parse_message(&mut iter)?,)),
                0x8D => ActionOperation::Version(()),
                0x8C => ActionOperation::ToggleMonitor(()),
                0x8B => ActionOperation::InitJoy(()),
                0x8A => ActionOperation::CancelLine(()),
                0x89 => ActionOperation::EchoLine(()),
                0x88 => ActionOperation::Pause(()),
                0x87 => ActionOperation::ShowMem(()),
                0x86 => ActionOperation::QuitV0(()),
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
                0x5E => ActionOperation::Drop((Self::parse_item(&mut iter)?,)),
                0x5D => ActionOperation::GetV((Self::parse_var(&mut iter)?,)),
                0x5C => ActionOperation::Get((Self::parse_item(&mut iter)?,)),
                0x5B => ActionOperation::Unblock(()),
                0x5A => ActionOperation::Block(Self::parse_num_num_num_num(&mut iter)?),
                0x59 => ActionOperation::ObserveBlocks((Self::parse_object(&mut iter)?,)),
                0x58 => ActionOperation::IgnoreBlocks((Self::parse_object(&mut iter)?,)),
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
                0x49 => ActionOperation::EndOfLoop(Self::parse_object_flag(&mut iter)?),
                0x47 => ActionOperation::StartCycling((Self::parse_object(&mut iter)?,)),
                0x46 => ActionOperation::StopCycling((Self::parse_object(&mut iter)?,)),
                0x45 => ActionOperation::Distance(Self::parse_object_object_var(&mut iter)?),
                0x44 => ActionOperation::ObserveObjs((Self::parse_object(&mut iter)?,)),
                0x43 => ActionOperation::IgnoreObjs((Self::parse_object(&mut iter)?,)),
                0x41 => ActionOperation::ObjectOnLand((Self::parse_object(&mut iter)?,)),
                //0x40 => ActionOperation::ObjectOnWater((Self::parse_object(&mut iter)?,)),
                0x3F => ActionOperation::SetHorizon((Self::parse_num(&mut iter)?,)),
                0x3E => ActionOperation::ObserveHorizon((Self::parse_object(&mut iter)?,)),
                0x3D => ActionOperation::IgnoreHorizon((Self::parse_object(&mut iter)?,)),
                0x3C => ActionOperation::ForceUpdate((Self::parse_object(&mut iter)?,)),
                0x3B => ActionOperation::StartUpdate((Self::parse_object(&mut iter)?,)),
                0x3A => ActionOperation::StopUpdate((Self::parse_object(&mut iter)?,)),
                0x39 => ActionOperation::GetPriority(Self::parse_object_var(&mut iter)?),
                0x38 => ActionOperation::ReleasePriority((Self::parse_object(&mut iter)?,)),
                //0x37 => ActionOperation::SetPriorityV(Self::parse_object_var(&mut iter)?),
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
                0x1B => ActionOperation::DiscardPic((Self::parse_var(&mut iter)?,)),
                0x1A => ActionOperation::ShowPic(()),
                0x19 => ActionOperation::DrawPic((Self::parse_var(&mut iter)?,)),
                0x18 => ActionOperation::LoadPic((Self::parse_var(&mut iter)?,)),
                0x17 => ActionOperation::CallV((Self::parse_var(&mut iter)?,)),
                0x16 => ActionOperation::Call((Self::parse_num(&mut iter)?,)),
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
            let is_goto= match op.action {
                ActionOperation::Goto(_) => true,
                _ => false,
            };

            let mut destination:TypeGoto = 0i16.into();
            match op.action {
                ActionOperation::Goto((g,)) | ActionOperation::If((_,g)) => {
                    let base_offset=offsets_rev.get(&(index+1)).unwrap();
                    destination = *base_offset+g;
                    if labels.contains_key(&destination) {
                        if labels[&destination].operation_offset != offsets[&destination] {
                            panic!("WUT!");
                        }
                        if is_goto {
                            labels.get_mut(&destination).unwrap().is_goto_destination=true;
                        } else {
                            labels.get_mut(&destination).unwrap().if_destination_cnt+=1;
                        }
                    } else {
                        labels.insert(destination, Label { is_goto_destination:is_goto, if_destination_cnt: if !is_goto {1} else {0}, operation_offset:offsets[&destination]});
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
        return Ok(LogicSequence { operations, labels });
    }

    pub fn get_operations(&self) -> &Vec<LogicOperation> {
        return &self.operations;
    }

    pub fn lookup_offset(&self,goto:&TypeGoto) -> Option<usize> {
        return match self.labels.get(goto) {
            Some(b) => Some(b.operation_offset),
            None => None,
        };
    }

    fn evaluate_condition_operation(state:&LogicState,op:&ConditionOperation) -> bool {
        return match op {
            ConditionOperation::EqualN((var,num)) => state.get_var(var) == state.get_num(num),
            ConditionOperation::EqualV((var1,var2)) => state.get_var(var1) == state.get_var(var2),
            ConditionOperation::LessN((var,num)) => state.get_var(var) < state.get_num(num),
            ConditionOperation::LessV((var1,var2)) => state.get_var(var1) < state.get_var(var2),
            ConditionOperation::GreaterN((var,num)) => state.get_var(var) > state.get_num(num),
            ConditionOperation::GreaterV((var1,var2)) => state.get_var(var1) > state.get_var(var2), 
            ConditionOperation::IsSet((flag,)) => state.get_flag(flag) == true,
            ConditionOperation::IsSetV(_) => todo!(),
            ConditionOperation::Has(_) =>  /* TODO */ false,
            ConditionOperation::PosN(_) => todo!(),
            ConditionOperation::Controller(_) => /* TODO */ false,
            ConditionOperation::HaveKey(_) => todo!(),
            ConditionOperation::Said(_) => /* TODO */ false,
            ConditionOperation::ObjInBox(_) => todo!(),
            ConditionOperation::RightPosN(_) => todo!(),
        }
    }

    fn evaluate_condition_or(state:&LogicState,cond:&Vec<LogicChange>) -> bool {
        let mut result = false;
        for a in cond {
            result = result | match a {
                LogicChange::Normal((op,)) => Self::evaluate_condition_operation(state,op),
                LogicChange::Not((op,)) => !Self::evaluate_condition_operation(state,op),
                _ => panic!("Should not occur i think {:?}", a),
            };
        }
        return result;
    }

    fn evaluate_condition(state:&LogicState,cond:&Vec<LogicChange>) -> bool {
        let mut result = true;
        for a in cond {
            result = result & match a {
                LogicChange::Normal((op,)) => Self::evaluate_condition_operation(state,op),
                LogicChange::Not((op,)) => !Self::evaluate_condition_operation(state,op),
                LogicChange::Or((or_block,)) => Self::evaluate_condition_or(state, or_block),
            };
            if result==false {  // Early out evaluation
                break;
            }
        }
        return result;
    }

    fn new_room(state:&mut LogicState,room:u8) {
        // Stop.update()
        //unanimate.all()
        //destroy all resources
        //player.control()
        //unblock()
        state.set_horizon(36);
        state.set_var(&TypeVar::from(1),state.get_var(&TypeVar::from(0)));
        state.set_var(&TypeVar::from(0), room);
        state.set_var(&TypeVar::from(4),0);
        state.set_var(&TypeVar::from(5),0);
        state.set_var(&TypeVar::from(9),0);
        state.set_var(&TypeVar::from(16),0);    // Should be ego view num
        //ego coords from var 2
        state.set_var(&TypeVar::from(2),0);
        state.set_flag(&TypeFlag::from(2),false);
        state.set_flag(&TypeFlag::from(5),true);
        // score<- var 3
    }


    fn interpret_instruction(&self,state:&mut LogicState,pc:&LogicExecutionPosition,action:&ActionOperation) -> Option<LogicExecutionPosition> {

        //println!("{:?}",state);
        println!("{:?}",action);

        match action {
            ActionOperation::If((condition,goto_if_false)) => if !Self::evaluate_condition(state,condition) { return Some(pc.jump(self,goto_if_false)); },
            ActionOperation::Goto((goto,)) => return Some(pc.jump(self, goto)),
            ActionOperation::Return(()) => return None,
            ActionOperation::Call((num,)) => return Some(LogicExecutionPosition {logic_file:state.get_num(num) as usize, program_counter: 0}),
            ActionOperation::CallV((var,)) => return Some(LogicExecutionPosition {logic_file:state.get_var(var) as usize, program_counter: 0}),
            ActionOperation::AssignN((var,num)) => state.set_var(var,state.get_num(num)),
            ActionOperation::AssignV((var1,var2)) => state.set_var(var1,state.get_var(var2)),
            ActionOperation::NewRoom((num,)) => { Self::new_room(state,state.get_num(num)); return None },
            ActionOperation::Reset((flag,)) => state.set_flag(flag, false),
            ActionOperation::ResetV((var,)) => { let flag=&TypeFlag::from(state.get_var(var)); state.set_flag(flag, false); },
            ActionOperation::AnimateObj((obj,)) => state.mut_object(obj).set_active(true),
            ActionOperation::LoadView((num,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadViewV((var,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadPic((var,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadLogic((num,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadSound((num,)) => {/* NO-OP-RAGI */},
            ActionOperation::SetView((obj,num)) => {let n=state.get_num(num); state.mut_object(obj).set_view(n); },
            ActionOperation::SetViewV((obj,var)) => {let n=state.get_var(var); state.mut_object(obj).set_view(n); },
            ActionOperation::ObserveObjs((obj,)) => state.mut_object(obj).set_observing(true),
            ActionOperation::LIndirectN((var,num)) => {let v = &TypeVar::from(state.get_var(var)); state.set_var(v,state.get_num(num)); },
            ActionOperation::Increment((var,)) => state.set_var(var,state.get_var(var).saturating_add(1)),
            ActionOperation::Decrement((var,)) => state.set_var(var,state.get_var(var).saturating_sub(1)),
            ActionOperation::GetPosN((obj,var1,var2)) => { state.set_var(var1, state.object(obj).get_x()); state.set_var(var2, state.object(obj).get_y()); },
            ActionOperation::StopCycling((obj,)) => state.mut_object(obj).set_cycling(false),
            ActionOperation::PreventInput(()) => state.set_input(false),
            ActionOperation::SetHorizon((num,)) => state.set_horizon(state.get_num(num)),
            ActionOperation::Position((obj,num1,num2)) => { let x=state.get_num(num1); let y=state.get_num(num2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y); },
            ActionOperation::SetPriority((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_priority(n); },
            ActionOperation::SetLoop((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_loop(n); },
            ActionOperation::SetCel((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_cel(n); },
            _ => panic!("TODO {:?}",action),
        }

        return Some(pc.next());
    }
    
    pub fn interpret_instructions(&self,state:&mut LogicState,pc:&LogicExecutionPosition,actions:&Vec<LogicOperation>) -> Option<LogicExecutionPosition> {
        return self.interpret_instruction(state, pc, &actions[pc.program_counter].action);
    }

}

#[derive(Copy,Clone)]
pub struct LogicExecutionPosition {
    logic_file:usize,
    program_counter:usize,
}

impl LogicExecutionPosition {
    pub fn new(file:usize,pc:usize) -> LogicExecutionPosition {
        return LogicExecutionPosition { logic_file: file, program_counter: pc };
    }

    pub fn next(&self) -> LogicExecutionPosition {
        return LogicExecutionPosition { logic_file: self.logic_file, program_counter: self.program_counter+1 };
    }

    pub fn jump(&self, sequence:&LogicSequence, goto:&TypeGoto) -> LogicExecutionPosition {
        return LogicExecutionPosition { logic_file: self.logic_file, program_counter: sequence.lookup_offset(goto).unwrap() }
    }

    pub fn is_call(&self,logic_file:usize) -> bool {
        return self.logic_file!=logic_file;
    }

    pub fn get_logic(&self) -> usize {
        return self.logic_file;
    }
}