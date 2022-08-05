use std::{collections::{HashMap, VecDeque}, hash::Hash, ops, fmt, fs};

use dir_resource::{ResourceDirectoryEntry, ResourceDirectory};
use fixed::{FixedU16, types::extra::U8, FixedI32};
use helpers::{Root, double_pic_width};
use itertools::Itertools;
use objects::{Objects, Object};
use picture::*;
use rand::{Rng, prelude::ThreadRng};
use view::{ViewResource, ViewCel, ViewLoop};
use volume::Volume;
use words::Words;

use strum_macros::IntoStaticStr;

pub const SCREEN_WIDTH_USIZE:usize = 320;
pub const SCREEN_HEIGHT_USIZE:usize = 200;

pub const OBJECT_EGO:TypeObject = type_object_from_u8(0);

pub const VAR_CURRENT_ROOM:TypeVar = type_var_from_u8(0);
pub const VAR_PREVIOUS_ROOM:TypeVar = type_var_from_u8(1);
pub const VAR_EGO_EDGE:TypeVar = type_var_from_u8(2);
pub const VAR_CURRENT_SCORE:TypeVar = type_var_from_u8(3);
pub const VAR_OBJ_TOUCHED_BORDER:TypeVar = type_var_from_u8(4);
pub const VAR_OBJ_EDGE:TypeVar = type_var_from_u8(5);
pub const VAR_EGO_MOTION_DIR:TypeVar = type_var_from_u8(6);
pub const VAR_MAXIMUM_SCORE:TypeVar = type_var_from_u8(7);
pub const VAR_FREE_PAGES:TypeVar = type_var_from_u8(8);
pub const VAR_MISSING_WORD:TypeVar = type_var_from_u8(9);
pub const VAR_TIME_DELAY:TypeVar = type_var_from_u8(10);
pub const VAR_SECONDS:TypeVar = type_var_from_u8(11);
pub const VAR_MINUTES:TypeVar = type_var_from_u8(12);
pub const VAR_HOURS:TypeVar = type_var_from_u8(13);
pub const VAR_DAYS:TypeVar = type_var_from_u8(14);

pub const VAR_EGO_VIEW:TypeVar = type_var_from_u8(16);

pub const VAR_CURRENT_KEY:TypeVar = type_var_from_u8(19);

pub const FLAG_EGO_IN_WATER:TypeFlag = type_flag_from_u8(0);

pub const FLAG_COMMAND_ENTERED:TypeFlag = type_flag_from_u8(2);
pub const FLAG_EGO_TOUCHED_SIGNAL:TypeFlag = type_flag_from_u8(3);
pub const FLAG_SAID_ACCEPTED_INPUT:TypeFlag = type_flag_from_u8(4);
pub const FLAG_ROOM_FIRST_TIME:TypeFlag = type_flag_from_u8(5);
pub const FLAG_RESTART_GAME:TypeFlag = type_flag_from_u8(6);

pub const FLAG_RESTORE_GAME:TypeFlag = type_flag_from_u8(12);

pub const FLAG_LEAVE_WINDOW_OPEN:TypeFlag = type_flag_from_u8(15);

type FP16=FixedU16<U8>;
type FP32=FixedI32<U8>;

#[derive(Debug)]
pub enum SpriteMotion {
    Normal,
    Wander,
    MoveObj,
    FollowEgo,
}

#[derive(Debug)]
pub enum SpriteCycle {
    Normal,
    Reverse,
    OneShot,
    OneShotReverse
}

#[derive(Debug)] // TODO revisit copy
pub struct Sprite {
    active:bool,    // object is processed
    moved:bool,     // object moved last tick
    frozen:bool,    // object ignores updates (animation/etc)
    visible:bool,   // draw to screen (draw/erase, to confirm if this is automated (sprite), or blit)
    motion:bool,    // movement is applied
    observing:bool,         // treats other objects as obstacles
    ignore_barriers:bool,   // ignores pixels of priority and block set with block_command
    ignore_horizon:bool,    // ignores horizon position during movement
    fixed_loop:bool,        // loop is not automatically determined
    restrict_to_land:bool,  // object is restricted to non priority 3 pixels
    restrict_to_water:bool, // object is restricted to priority 3 pixels
    cycle:bool,     // cycle loop automatically
    cycle_kind:SpriteCycle,
    motion_kind:SpriteMotion,
    direction:u8,   // current direction of travel (0-stop, 1-N, 2-NE, 3-E, ... 8-NW)
    view:u8,
    cloop:u8,
    cel:u8,
    x:FP16,           // bottom left corner
    y:FP16,
    priority:u8,
    cycle_flag:TypeFlag,
    move_flag:TypeFlag,
    ex:FP16,
    ey:FP16,
    step_size:FP16,
    step_time:u8,
    step_cnt:u8,
    cycle_time:u8,
    cycle_cnt:u8,
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite { 
            active: false, 
            moved: false,
            frozen: false,
            visible: false,
            motion: false,
            observing: false, 
            ignore_barriers: false,
            ignore_horizon: false,
            fixed_loop: false,
            restrict_to_land: false,
            restrict_to_water: false,
            cycle: true, 
            cycle_kind: SpriteCycle::Normal,
            motion_kind: SpriteMotion::Normal,
            direction: 0,
            view: 0, 
            cloop: 0,
            cel: 0,
            x:FP16::from_num(0), 
            y:FP16::from_num(0),
            priority:0,
            cycle_flag:TypeFlag::from(0),
            move_flag: TypeFlag::from(0),
            ex: FP16::from_num(0),
            ey: FP16::from_num(0),
            step_size: FP16::from_bits(4<<6),
            step_time: 1,
            step_cnt: 0,
            cycle_time: 1,
            cycle_cnt: 0,
        }
    }

    pub fn get_x(&self) -> u8 {
        self.x.to_num()
    }

    pub fn get_y(&self) -> u8 {
        self.y.to_num()
    }

    pub fn get_direction(&self) -> u8 {
        self.direction
    }

    pub fn get_view(&self) -> u8 {
        self.view
    }

    pub fn get_loop(&self) -> u8 {
        self.cloop
    }

    pub fn get_cel(&self) -> u8 {
        self.cel
    }

    pub fn get_visible(&self) -> bool {
        self.visible
    }
    
    pub fn is_restricted_by_blocks(&self) -> bool {
        self.observing
    }

    pub fn is_restricted_to_land(&self) -> bool {
        self.restrict_to_land
    }

    pub fn is_restricted_to_water(&self) -> bool {
        self.restrict_to_water
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn has_moved(&self) -> bool {
        self.moved
    }

    pub fn get_x_fp16(&self) -> FP16 {
        self.x
    }
    
    pub fn get_y_fp16(&self) -> FP16 {
        self.y
    }

    pub fn get_step_size(&self) -> FP16 {
        self.step_size
    }

    pub fn get_end_x(&self) -> FP16 {
        self.ex
    }
    
    pub fn get_end_y(&self) -> FP16 {
        self.ey
    }
    
    pub fn get_motion_kind(&self) -> &SpriteMotion {
        &self.motion_kind
    }

    pub fn get_priority(&self) -> u8 {
        if self.priority == 0 {
            // Automatic priority
            let y:u8 = self.y.to_num();
            match y {
                0..=47    => 4,
                48..=59   => 5,
                60..=71   => 6,
                72..=83   => 7,
                84..=95   => 8,
                96..=107  => 9,
                108..=119 => 10,
                120..=131 => 11,
                132..=143 => 12,
                144..=155 => 13,
                156..=167 => 14,
                _         => 15,
            }
        } else {
            self.priority
        }
    }

    pub fn distance(&self,other:&Sprite) -> u8 {
        if !self.get_visible() || !other.get_visible() {
            return 255;
        }

        let x1:i16=self.get_x().into();
        let x2:i16=other.get_x().into();
        let y1:i16=self.get_y().into();
        let y2:i16=other.get_y().into();
        
        let d = (x1-x2).abs().wrapping_add((y1-y2).abs());

        d as u8
    }

    pub fn should_step(&mut self) -> bool {
        self.step_cnt+=1;
        if self.step_cnt >= self.step_time 
        {
            self.step_cnt=0;
            true
        }
        else {
            false
        }
    }
    
    pub fn should_cycle(&mut self) -> bool {
        self.cycle_cnt+=1;
        if self.cycle_cnt >= self.cycle_time 
        {
            self.cycle_cnt=0;
            true
        }
        else {
            false
        }
    }

    pub fn set_active(&mut self,b:bool) {
        self.active=b;
    }
    
    pub fn set_frozen(&mut self,b:bool) {
        self.frozen=b;
    }

    pub fn set_visible(&mut self,b:bool) {
        self.visible=b;
    }

    pub fn set_observing(&mut self,b:bool) {
        self.observing=b;
    }

    pub fn set_ignore_barriers(&mut self,b:bool) {
        self.ignore_barriers=b;
    }

    pub fn set_ignore_horizon(&mut self,b:bool) {
        self.ignore_horizon=b;
    }

    pub fn set_step_size(&mut self,s:u8) {
        if s!=0 {
            self.step_size=FP16::from_bits((s as u16)<<6);
        }
    }

    pub fn set_direction(&mut self,d:u8) {
        self.direction=d;
    }

    pub fn set_cycling(&mut self,b:bool) {
        self.cycle=b;
    }

    pub fn set_step_time(&mut self,n:u8) {
        self.step_time=n;
        self.step_cnt=0;
    }
    
    pub fn set_cycle_time(&mut self,n:u8) {
        self.cycle_time=n;
        self.cycle_cnt=0;
    }


    pub fn set_x(&mut self,n:u8) {
        self.x = FP16::from_num(n);
    }
    
    pub fn set_y(&mut self,n:u8) {
        self.y = FP16::from_num(n);
    }
 
    pub fn set_x_fp16(&mut self,n:FP16) {
        self.x = n;
    }
    
    pub fn set_y_fp16(&mut self,n:FP16) {
        self.y = n;
    }

    pub fn set_priority(&mut self,n:u8) {
        self.priority = n;
    }

    pub fn set_priority_auto(&mut self) {
        self.priority = 0;
    }
   
    pub fn set_view(&mut self, view:u8) {
        self.view = view;
    }

    pub fn set_loop(&mut self,n:u8) {
        self.cloop = n;
    }
    
    pub fn set_cel(&mut self,n:u8) {
        self.cel = n;
    }

    pub fn set_fixed_loop(&mut self,b:bool) {
        self.fixed_loop = b;
    }

    pub fn set_one_shot(&mut self,f:&TypeFlag) {
        self.cycle_kind=SpriteCycle::OneShot;
        self.cycle_flag = *f;
        self.cycle=true;
    }

    pub fn set_one_shot_reverse(&mut self,f:&TypeFlag) {
        self.cycle_kind=SpriteCycle::OneShotReverse;
        self.cycle_flag = *f;
        self.cycle=true;
    }

    pub fn end_one_shot(&mut self) {
        self.cycle=false;
        self.cycle_kind=SpriteCycle::Normal;
    }

    pub fn set_moved(&mut self,b:bool) {
        self.moved=b;
    }

    pub fn set_move(&mut self,x:u8,y:u8,s:u8,f:&TypeFlag) {
        self.set_enable_motion(true);   // to confirm
        self.set_frozen(false);         // to confirm
        self.motion_kind=SpriteMotion::MoveObj;
        self.ex=FP16::from_num(x);
        self.ey=FP16::from_num(y);
        self.move_flag= *f;
        self.set_step_size(s)
    }

    pub fn set_follow(&mut self,s:u8,f:&TypeFlag) {
        self.set_enable_motion(true);   // to confirm
        self.set_frozen(false);         // to confirm
        self.motion_kind=SpriteMotion::FollowEgo;
        self.move_flag= *f;
        self.set_step_size(s)
    }

    pub fn clear_move(&mut self) {
        self.motion_kind=SpriteMotion::Normal;
    }

    pub fn adjust_x_via_delta(&mut self,dx:u8) {
        let t = FP16::from_num(dx);
        self.x = self.x.wrapping_add(t);
    }

    pub fn adjust_y_via_delta(&mut self,dy:u8) {
        let t = FP16::from_num(dy);
        self.y = self.y.wrapping_add(t);
    }

    pub fn reset(&mut self) {
        *self=Sprite::new();
    }

    pub fn set_restrict_to_water(&mut self) {
        self.restrict_to_land=false;
        self.restrict_to_water=true;
    }

    pub fn set_restrict_to_land(&mut self) {
        self.restrict_to_land=true;
        self.restrict_to_water=false;
    }

    pub fn set_wander(&mut self) {
        self.motion_kind=SpriteMotion::Wander;
        self.set_enable_motion(true);   // to confirm
    }

    pub fn set_normal_motion(&mut self) {
        self.motion_kind=SpriteMotion::Normal;
        self.set_enable_motion(true);   // to confirm
    }

    pub fn set_enable_motion(&mut self,b:bool) {
        self.motion=b;
    }
}

pub struct GameResources
{
    pub objects:Objects,
    pub words:Words,
    pub views:HashMap<usize,ViewResource>,
    pub pictures:HashMap<usize,PictureResource>,
    pub logic:HashMap<usize,LogicResource>,
    pub font:Vec<u8>,
}

impl GameResources {
    pub fn new (base_path:&'static str,version:&str) -> Result<GameResources,String> {

        // hack for font
        let font = fs::read("../images/BM.PSF").unwrap();

        let root = Root::new(base_path);
    
        let mut volumes:HashMap<u8,Volume>=HashMap::new();

        let dir = ResourceDirectory::new(root.read_data_or_default("VIEWDIR").into_iter()).unwrap();

        let mut views:HashMap<usize,ViewResource> = HashMap::new();
        views.reserve(256);
        for (index,entry) in dir.into_iter().enumerate() {
            if !entry.empty() {
                if let std::collections::hash_map::Entry::Vacant(e) =volumes.entry(entry.volume) {
                    let bytes = root.read_data_or_default(format!("VOL.{}", entry.volume).as_str());
                    e.insert(Volume::new(bytes.into_iter())?);
                }
                views.insert(index, ViewResource::new(&volumes[&entry.volume],&entry)?);
            }
        }
        views.shrink_to_fit();

        let dir = ResourceDirectory::new(root.read_data_or_default("PICDIR").into_iter()).unwrap();

        let mut pictures:HashMap<usize,PictureResource> = HashMap::new();
        pictures.reserve(256);
        for (index,entry) in dir.into_iter().enumerate() {
            if !entry.empty() {
                if let std::collections::hash_map::Entry::Vacant(e) = volumes.entry(entry.volume) {
                    let bytes = root.read_data_or_default(format!("VOL.{}", entry.volume).as_str());
                    e.insert(Volume::new(bytes.into_iter())?);
                }
                pictures.insert(index, PictureResource::new(&volumes[&entry.volume],&entry)?);
            }
        }
        pictures.shrink_to_fit();

        let dir = ResourceDirectory::new(root.read_data_or_default("LOGDIR").into_iter()).unwrap();

        let mut logic:HashMap<usize,LogicResource> = HashMap::new();
        logic.reserve(256);
        for (index,entry) in dir.into_iter().enumerate() {
            if !entry.empty() {
                if let std::collections::hash_map::Entry::Vacant(e) = volumes.entry(entry.volume) {
                    let bytes = root.read_data_or_default(format!("VOL.{}", entry.volume).as_str());
                    e.insert(Volume::new(bytes.into_iter())?);
                }
                logic.insert(index, LogicResource::new(&volumes[&entry.volume],&entry,version)?);
            }
        }
        logic.shrink_to_fit();

        return Ok(GameResources {
            words : Words::new(root.read_data_or_default("WORDS.TOK").into_iter())?,
            objects: Objects::new(&root.read_data_or_default("OBJECT"))?,
            views,
            pictures,
            logic,
            font,
        });
    }
}

pub struct TextWindow {
    pub x0:u16,
    pub x1:u16,
    pub y0:u8,
    pub y1:u8,
}

impl TextWindow {
    pub fn new() -> TextWindow {
        TextWindow {x0:0,x1:0,y0:0,y1:0}
    }

    pub fn is_empty(&self) -> bool {
        self.x0==self.x1 || self.y0==self.y1
    }
}

pub struct LogicState {
    rng:ThreadRng,
    new_room:u8,
    text_mode:bool,
    input:bool,
    ego_player_control:bool,
    status_visible:bool,
    horizon:u8,
    flag:[bool;256],
    var:[u8;256],
    objects:[Sprite;256],   // overkill, todo add list of active
    string:[String;256],    // overkill
    words:[u16;256],        // overkill
    logic_start:[usize;256],
    item_location:[u8;256],

    num_string:String,
    prompt:char,
    parsed_input_string:String,

    ink:u8,     // colours for display/get_string/get_num
    paper:u8,

    windows:[TextWindow;2], // Holds the co-ordinates of the message window last drawn (and item from show.obj)
    displayed:String,

    //input
    key_len:usize,
    key_buffer:[u8;256],

    // video
    video_buffer:[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],
    priority_buffer:[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],

    back_buffer:[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],
    post_sprites:[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],

    text_buffer:[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],
    final_buffer:[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],
}

impl Default for LogicState {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicState {

    pub fn new() -> LogicState {
        LogicState {
            rng:rand::thread_rng(),
            new_room: 0,
            text_mode:false,
            input: false,
            ego_player_control: true,
            status_visible: false,
            horizon: 0,
            flag: [false;256],
            var: [0u8;256],
            objects: [();256].map(|_| Sprite::new()),
            string: [();256].map(|_| String::new()),
            words: [0u16;256],
            item_location: [0u8;256],
            logic_start: [0usize;256],
            num_string: String::from(""),
            prompt:'_',
            parsed_input_string: String::from(""),
            windows:[();2].map(|_| TextWindow::new()),
            displayed: String::from(""),
            ink:15,
            paper:0,
            key_len:0,
            key_buffer:[0;256],
            video_buffer:[15;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],
            priority_buffer:[4;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],
            back_buffer:[0;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],
            post_sprites:[0;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],
            text_buffer:[255u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],
            final_buffer:[0;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE],
        }
    }

    pub fn initialise_rooms(&mut self,items:&Vec<Object>) {
        for (idx,i) in items.iter().enumerate() {
            self.item_location[idx]=i.start_room;
        }
    }

    pub fn get_parsed_word_num(&self,num:u8) -> String {
        for (idx,w) in self.parsed_input_string.split(' ').enumerate() {
            if idx==(num as usize)-1 {
                return w.to_string();
            }
        } 
        String::from("??")
    }

    pub fn get_flags(&self) -> impl Iterator<Item = bool> {
        self.flag.into_iter()
    }

    pub fn get_flag(&self,f:&TypeFlag) -> bool {
        self.flag[f.value as usize]
    }

    pub fn get_vars(&self) -> impl Iterator<Item = u8> {
        self.var.into_iter()
    }

    pub fn get_var(&self,v:&TypeVar) -> u8 {
        self.var[v.value as usize]
    }

    pub fn get_num(&self,v:&TypeNum) -> u8 {
        v.value
    }

    pub fn get_logic_start(&self,l:u8) -> usize {
        self.logic_start[l as usize]
    }

    pub fn get_controller(&self,c:&TypeController) -> u8 {
        c.value
    }

    pub fn get_new_room(&self) -> u8 {
        self.new_room
    }

    pub fn get_item_room(&self,item:&TypeItem) -> u8 {
        self.item_location[item.value as usize]
    }

    pub fn get_message(&self,m:&TypeMessage) -> u8 {
        m.value
    }

    pub fn get_strings(&self) -> impl Iterator<Item = &String> {
        self.string.iter()
    }

    pub fn get_num_string(&self) -> &String {
        &self.num_string
    }

    pub fn get_string(&self,s:&TypeString) -> &String {
        &self.string[s.value as usize]
    }

    pub fn get_prompt(&self) -> char {
        self.prompt
    }

    pub fn get_ink(&self) -> u8 {
        self.ink
    }

    pub fn get_paper(&self) -> u8 {
        self.paper
    }

    pub fn is_text_mode(&self) -> bool {
        self.text_mode
    }

    pub fn is_ego_player_controlled(&self) -> bool {
        self.ego_player_control
    }

    pub fn is_input_enabled(&self) -> bool {
        self.input && !self.text_mode
    }
 
    pub fn check_said(&mut self,to_check:&Vec<TypeWord>) -> bool {
        if !self.get_flag(&FLAG_COMMAND_ENTERED) || self.get_flag(&FLAG_SAID_ACCEPTED_INPUT) {
            return false;
        }

        for (index,word) in to_check.iter().enumerate() {
            // Match any word, but out of words to match against
            if word.value == 1 && self.words[index]==0 {
                return false;
            }
            // Match remainder of input
            if word.value == 9999 {
                break;
            }
            // Word does not match
            if word.value != self.words[index] {
                return false;
            }
        }
        self.set_flag(&FLAG_SAID_ACCEPTED_INPUT, true);
        true
    }

    pub fn mut_num_string(&mut self) -> &mut String {
        &mut self.num_string
    }

    pub fn get_mut_string(&mut self,s:&TypeString) -> &mut String {
        &mut self.string[s.value as usize]
    }

    pub fn get_random(&mut self,start:&TypeNum,end:&TypeNum) -> u8 {
        self.rng.gen_range(self.get_num(start)..self.get_num(end))
    }

    pub fn set_logic_start(&mut self,pos:&LogicExecutionPosition) {
        self.logic_start[pos.logic_file]=pos.program_counter;
    }

    pub fn clear_logic_start(&mut self,pos:&LogicExecutionPosition) {
        self.logic_start[pos.logic_file]=0;
    }

    pub fn set_var(&mut self,v:&TypeVar,n:u8) {
        self.var[v.value as usize] = n;
    }

    pub fn set_flag(&mut self,f:&TypeFlag,n:bool) {
        self.flag[f.value as usize] = n;
    }

    pub fn set_string(&mut self,s:&TypeString,m:&str) {
        self.string[s.value as usize] = m.to_owned();
    }
    
    pub fn set_prompt(&mut self,m:&str) {
        if m.len()>0 {
            self.prompt = m.chars().next().unwrap();
        } else {
            self.prompt=' ';
        }
    }

    pub fn set_ink(&mut self,ink:u8) {
        self.ink=ink;
    }

    pub fn set_paper(&mut self,paper:u8) {
        self.paper=paper;
    }

    pub fn set_input(&mut self,b:bool) {
        self.input = b;
    }
    
    pub fn set_horizon(&mut self,h:u8) {
        self.horizon = h;
    }

    pub fn reset_new_room(&mut self) {
        self.new_room = 0;
    }

    pub fn set_player_control(&mut self) {
        self.ego_player_control=true;
    }
    
    pub fn set_program_control(&mut self) {
        self.ego_player_control=false;
    }

    pub fn set_item_location(&mut self,item:&TypeItem,loc:u8) {
        self.item_location[item.value as usize]=loc;
    }

    pub fn set_new_room(&mut self,r:u8) {
        self.new_room = r;
    }

    pub fn set_text_mode(&mut self,b:bool) {
        self.text_mode=b;
        self.text_buffer = [255u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE];
    }

    pub fn set_status_visible(&mut self,b:bool) {
        self.status_visible=b;
    }

    pub fn object(&self,o:&TypeObject) -> &Sprite {
        &self.objects[o.value as usize]
    }

    pub fn mut_object(&mut self,o:&TypeObject) -> &mut Sprite {
        &mut self.objects[o.value as usize]
    }

    pub fn active_objects_indices(&self) -> impl Iterator<Item = usize>{
        let t_indices:Vec<usize> = (0..self.objects.len())
            .filter(|b| self.object(&type_object_from_u8(*b as u8)).is_active())
            .collect_vec();
        t_indices.into_iter()
    }
    
    pub fn active_objects_indices_sorted_y(&self) -> impl Iterator<Item = usize> {
        let t_indices:Vec<usize> = (0..self.objects.len())
            .filter(|b| self.object(&type_object_from_u8(*b as u8)).is_active())
            .sorted_unstable_by(|a,b| Ord::cmp(&self.object(&type_object_from_u8(*a as u8)).get_y_fp16(),&self.object(&type_object_from_u8(*b as u8)).get_y_fp16()))
            .collect_vec();
        t_indices.into_iter()
    }
    
    pub fn mut_active_objects(&mut self) -> impl Iterator<Item = (usize,&mut Sprite)> {
        (0..self.objects.len()).zip(self.objects.iter_mut()).filter(|(_,b)| b.active)
    }

    pub fn picture(&self) -> &[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE] {
        &self.video_buffer
    }
    
    pub fn mut_picture(&mut self) -> &mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE] {
        &mut self.video_buffer
    }
    
    
    pub fn priority(&self) -> &[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE] {
        &self.priority_buffer
    }

    pub fn back_buffer(&self) -> &[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE] {
        &self.back_buffer
    }

    pub fn text_buffer(&self) -> &[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE] {
        &self.text_buffer
    }

    pub fn screen_buffer(&self) -> &[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE] {
        &self.post_sprites
    }

    pub fn final_buffer(&self) -> &[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE] {
        &self.final_buffer
    }

    pub fn render_final_buffer(&mut self) {
        if self.text_mode {
            self.final_buffer = [0u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE];
        } else {
            self.final_buffer = self.post_sprites;
        }

        // Now combine the text buffer
        for i in 0..SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE {
            if self.text_buffer[i]!=255 {
                self.final_buffer[i]=self.text_buffer[i];
            }
        }
    }

    pub fn clear_keys(&mut self) {
        self.key_len=0;
    }

    pub fn key_pressed(&mut self,ascii_code:u8) {
        if self.key_len<256 {
            self.key_buffer[self.key_len]=ascii_code;
            self.key_len+=1;
        }
    }

}

impl fmt::Debug for LogicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogicState")
        .field("new_room", &self.new_room)
        .field("input", &self.input)
        .field("ego_player_control", &self.ego_player_control)
        .field("status_visible",&self.status_visible)
        .field("horizon", &self.horizon)
        .field("flag", &self.flag)
        .field("var", &self.var)
        .field("objects", &self.objects)
        .field("string", &self.string)
        .field("text_mode",&self.text_mode)
        .field("key_buffer",&self.key_buffer)
        .finish()
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

const fn type_var_from_u8(n:u8) -> TypeVar {
    TypeVar {value:n }
}

const fn type_object_from_u8(n:u8) -> TypeObject {
    TypeObject {value:n }
}

const fn type_flag_from_u8(n:u8) -> TypeFlag {
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
    PutV ((TypeVar,TypeVar)),
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
    ClearTextRect((TypeNum,TypeNum,TypeNum,TypeNum,TypeNum)),
    SetMenu((TypeMessage,)),
    SetMenuMember((TypeMessage,TypeController)),
    SubmitMenu(()),
    DisableMember((TypeController,)),
    MenuInput(()),
    CloseWindow(()),
    ShowObjV((TypeVar,)),
    MulN((TypeVar,TypeNum)),
    MulV((TypeVar,TypeVar)),
    Goto((TypeGoto,)),
    If((Vec<LogicChange>,TypeGoto)),
}

impl LogicMessages {
    fn new(text_slice: &[u8]) -> Result<LogicMessages,&'static str> {
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
    pub fn new(volume:&Volume, entry: &ResourceDirectoryEntry, version:&str) -> Result<LogicResource, &'static str> {

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
        let logic_sequence = LogicSequence::new(logic_slice,version).expect("fsjkdfhksdjf");

        Ok(LogicResource {logic_sequence, logic_messages})
    }

    pub fn get_logic_sequence(&self) -> &LogicSequence {
        &self.logic_sequence
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
            ActionOperation::ObserveBlocks(a) => Self::param_dis_object(&a.0),
            ActionOperation::Get(a) |
            ActionOperation::Drop(a) => Self::param_dis_item(&a.0, items),
            ActionOperation::Print(a) |
            ActionOperation::SetMenu(a) |
            ActionOperation::SetCursorChar(a) |
            ActionOperation::Log(a) |
            ActionOperation::SetGameID(a) => self.param_dis_message(&a.0),
            ActionOperation::Parse(a) => Self::param_dis_string(&a.0),
            ActionOperation::DisableMember(a) => Self::param_dis_controller(&a.0),
            ActionOperation::SetTextAttribute(a) => format!("{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1)),
            ActionOperation::Sound(a) => format!("{},{}",Self::param_dis_num(&a.0),Self::param_dis_flag(&a.1)),
            ActionOperation::AddN(a) |
            ActionOperation::SubN(a) |
            ActionOperation::LIndirectN(a) |
            ActionOperation::MulN(a) |
            ActionOperation::AssignN(a) => format!("{},{}",Self::param_dis_var(&a.0),Self::param_dis_num(&a.1)),
            ActionOperation::AddV(a) |
            ActionOperation::SubV(a) |
            ActionOperation::GetRoomV(a) |
            ActionOperation::PutV(a) |
            ActionOperation::LIndirectV(a) |
            ActionOperation::RIndirect(a) |
            ActionOperation::MulV(a) |
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
            ActionOperation::PrintAtV0(a) => format!("{},{},{}",self.param_dis_message(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2)),
            ActionOperation::Block(a) => format!("{},{},{},{}",Self::param_dis_num(&a.0),Self::param_dis_num(&a.1),Self::param_dis_num(&a.2),Self::param_dis_num(&a.3)),
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

    fn new(logic_slice: &[u8],version:&str) -> Result<LogicSequence,&'static str> {

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
                0xA9 => ActionOperation::CloseWindow(()),
                0xA6 => ActionOperation::MulV(Self::parse_var_var(&mut iter)?),
                0xA5 => ActionOperation::MulN(Self::parse_var_num(&mut iter)?),
                0xA2 => ActionOperation::ShowObjV((Self::parse_var(&mut iter)?,)),
                0xA1 => ActionOperation::MenuInput(()),
                0xA0 => ActionOperation::DisableMember((Self::parse_controller(&mut iter)?,)),
                0x9E => ActionOperation::SubmitMenu(()),
                0x9D => ActionOperation::SetMenuMember(Self::parse_message_controller(&mut iter)?),
                0x9C => ActionOperation::SetMenu((Self::parse_message(&mut iter)?,)),
                0x9A => ActionOperation::ClearTextRect(Self::parse_num_num_num_num_num(&mut iter)?),
                0x97 => if version == "2.089" || version == "2.272" {ActionOperation::PrintAtV0(Self::parse_message_num_num(&mut iter)?) } else if version=="2.440" {ActionOperation::PrintAtV1(Self::parse_message_num_num_num(&mut iter)?)} else {panic!("DAMN")},
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
                0x86 => if version == "2.089" { ActionOperation::QuitV0(()) } else { ActionOperation::QuitV1((Self::parse_num(&mut iter)?,))},  // Check me, i`m not sure only 2.089 for V0.. see KQI
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
                0x60 => ActionOperation::PutV(Self::parse_var_var(&mut iter)?),
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

    pub fn get_operations(&self) -> &Vec<LogicOperation> {
        &self.operations
    }

    pub fn lookup_offset(&self,goto:&TypeGoto) -> Option<usize> {
        self.labels.get(goto).map(|b| b.operation_offset)
    }

    fn evaluate_condition_operation(resources:&GameResources,state:&mut LogicState,op:&ConditionOperation,need_tick:&mut bool) -> bool {
        match op {
            ConditionOperation::EqualN((var,num)) => state.get_var(var) == state.get_num(num),
            ConditionOperation::EqualV((var1,var2)) => state.get_var(var1) == state.get_var(var2),
            ConditionOperation::LessN((var,num)) => state.get_var(var) < state.get_num(num),
            ConditionOperation::LessV((var1,var2)) => state.get_var(var1) < state.get_var(var2),
            ConditionOperation::GreaterN((var,num)) => state.get_var(var) > state.get_num(num),
            ConditionOperation::GreaterV((var1,var2)) => state.get_var(var1) > state.get_var(var2), 
            ConditionOperation::IsSet((flag,)) => state.get_flag(flag),
            ConditionOperation::IsSetV(_) => todo!(),
            ConditionOperation::Has((item,)) => state.get_item_room(item)==255,
            ConditionOperation::ObjInRoom((item,var)) => { let n=state.get_var(var); state.get_item_room(item)==n },
            ConditionOperation::PosN((obj,num1,num2,num3,num4)) => is_left_edge_in_box(resources,state,obj,num1,num2,num3,num4),
            ConditionOperation::Controller(_) => /* TODO */ false,
            ConditionOperation::HaveKey(_) => {
                // Can lock up completely as often used like so :
                //recheck:
                //if !HaveKey() {
                //    goto recheck;
                //}
                // So for now, we let interpretter exit if false would be returned
                let key_pressed = state.key_len>0;
                state.clear_keys();
                *need_tick|=!key_pressed;
                key_pressed
            },
            ConditionOperation::Said((w,)) => state.check_said(w),
            ConditionOperation::CompareStrings(_) => todo!(),
            ConditionOperation::ObjInBox((obj,num1,num2,num3,num4)) => is_left_and_right_edge_in_box(resources,state,obj,num1, num2,num3,num4),
            ConditionOperation::RightPosN((obj,num1,num2,num3,num4)) => is_right_edge_in_box(resources,state,obj,num1,num2,num3,num4),
        }
    }

    fn evaluate_condition_or(resources:&GameResources,state:&mut LogicState,cond:&Vec<LogicChange>,need_tick:&mut bool) -> bool {
        let mut result = false;
        for a in cond {
            result |= match a {
                LogicChange::Normal((op,)) => Self::evaluate_condition_operation(resources,state,op,need_tick),
                LogicChange::Not((op,)) => !Self::evaluate_condition_operation(resources,state,op,need_tick),
                _ => panic!("Should not occur i think {:?}", a),
            };
        }
        result
    }

    fn evaluate_condition(resources:&GameResources,state:&mut LogicState,cond:&Vec<LogicChange>,need_tick:&mut bool) -> bool {
        let mut result = true;
        for a in cond {
            result &= match a {
                LogicChange::Normal((op,)) => Self::evaluate_condition_operation(resources,state,op,need_tick),
                LogicChange::Not((op,)) => !Self::evaluate_condition_operation(resources,state,op,need_tick),
                LogicChange::Or((or_block,)) => Self::evaluate_condition_or(resources,state, or_block,need_tick),
            };
            if !result {  // Early out evaluation
                break;
            }
        }
        result
    }

    pub fn new_room(resources:&GameResources,state:&mut LogicState,room:u8) {
        state.text_buffer = [255u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE];

        // Stop.update()
        //unanimate.all()
        for (_,obj) in state.mut_active_objects() {
            obj.set_active(false);
            obj.set_visible(false);
            obj.set_normal_motion();
            obj.set_direction(0);
            obj.set_cycling(true);
            obj.set_priority_auto();
            //obj.reset();//  (may not be needed)
        }
        //destroy all resources
        state.set_player_control();
        //unblock()
        state.set_horizon(36);
        state.set_var(&VAR_PREVIOUS_ROOM,state.get_var(&VAR_CURRENT_ROOM));
        state.set_var(&VAR_CURRENT_ROOM, room);
        state.set_var(&VAR_OBJ_TOUCHED_BORDER,0);
        state.set_var(&VAR_OBJ_EDGE,0);
        state.set_var(&VAR_MISSING_WORD,0);
        state.set_var(&VAR_EGO_VIEW,state.object(&OBJECT_EGO).get_view());
        
        let c = usize::from(state.object(&OBJECT_EGO).get_cel());
        let cels = get_cells_clamped(resources, state.object(&OBJECT_EGO));
        let cell = &cels[c];

        match state.get_var(&VAR_EGO_EDGE) {
            0 => {},
            1 => state.mut_object(&OBJECT_EGO).set_y(PIC_HEIGHT_U8-1),
            2 => state.mut_object(&OBJECT_EGO).set_x(0),
            3 => state.mut_object(&OBJECT_EGO).set_y(36+cell.get_height()),
            4 => state.mut_object(&OBJECT_EGO).set_x(PIC_WIDTH_U8-cell.get_width()),
            _ => panic!("Invalid edge in EGO EDGE"),
        }

        state.set_var(&VAR_EGO_EDGE,0);
        state.set_flag(&FLAG_COMMAND_ENTERED,false);
        state.set_flag(&FLAG_ROOM_FIRST_TIME,true);
        // score<- var 3
    }

    fn decode_message_from_resource(&self,state:&LogicState,resources:&GameResources,file:usize,message:&TypeMessage) -> String {
        let mut new_string=String::from("");
        let mut c_state = 0;
        let mut n_kind = b' ';
        let mut num = 0;
        for c in resources.logic[&file].logic_messages.strings[state.get_message(message) as usize].bytes() {
            match c_state {
                0 => if c == b'%' { c_state=1; } else { new_string.push(c as char); },
                1 => match c {
                    b'v' | b'm' | b'o' | b'w' | b's' | b'g'  => { n_kind=c; num=0; c_state=2; },
                    _ => todo!(),
                },
                2 => if c>=b'0' && c<=b'9' { num*=10; num+=c-b'0'; } else 
                {
                    new_string.push_str(match n_kind {
                        b'v' => state.get_var(&TypeVar::from(num)).to_string(),
                        b'm' => self.decode_message_from_resource(state, resources, file, &TypeMessage::from(num)),
                        b'o' => resources.objects.objects[num as usize].name.clone(),
                        b'w' => state.get_parsed_word_num(num),
                        b's' => state.get_string(&TypeString::from(num)).clone(),
                        b'g' => self.decode_message_from_resource(state, resources, 0, &TypeMessage::from(num)),
                        _ => todo!(),
                    }.as_str());
                    new_string.push(c as char); c_state=0; }
                _ => todo!(),
            }
        }
        new_string
    }

    fn interpret_instruction(&self,resources:&GameResources,state:&mut LogicState,pc:&LogicExecutionPosition,action:&ActionOperation) -> Option<LogicExecutionPosition> {

        match action {
            // Not complete
            ActionOperation::Sound((_num,flag)) => /* TODO RAGI  - for now, just pretend sound finished*/ {println!("TODO : Sound@{}",pc); state.set_flag(flag,true);},
            ActionOperation::StopSound(()) => /* TODO RAGI - for now, since we complete sounds straight away, does nothing */ {println!("TODO : StopSound@{}",pc);},
            ActionOperation::SetGameID((m,)) => /* TODO RAGI - if needed */{let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); println!("TODO : SetGameID@{} {:?}",pc,m);},
            ActionOperation::ConfigureScreen((a,b,c)) => /* TODO RAGI */ { println!("TODO : ConfigureScreen@{} {:?},{:?},{:?}",pc,a,b,c);},
            ActionOperation::SetKey((a,b,c)) => /* TODO RAGI */ { println!("TODO : SetKey@{} {:?},{:?},{:?}",pc,a,b,c);},
            ActionOperation::SetMenu((m,)) => /* TODO RAGI */ { let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); println!("TODO : SetMenu@{} {}",pc,m); },
            ActionOperation::SetMenuMember((m,c)) => /* TODO RAGI */{ let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); println!("TODO : SetMenuMember@{} {} {}",pc,m,state.get_controller(c)); },
            ActionOperation::SubmitMenu(()) => /* TODO RAGI */ { println!("TODO : SubmitMenu@{}",pc)},
            ActionOperation::TraceInfo((num1,num2,num3)) => /* TODO RAGI */ { println!("TODO : TraceInfo@{} {} {} {}",pc,state.get_num(num1),state.get_num(num2),state.get_num(num3)); }
            ActionOperation::DisableMember((c,)) => /* TODO RAGI */ println!("TODO : DisableMember@{} {}",pc, state.get_controller(c)),
            ActionOperation::CancelLine(()) => /* TODO RAGI */ println!("TODO : CancelLine@{}",pc),
            ActionOperation::ForceUpdate((o,)) => /* TODO RAGI */ println!("TODO : ForceUpdate@{} {:?}",pc,o),
            ActionOperation::ShakeScreen((num,)) => /* TODO RAGI */ println!("TODO : ShakeScreen@{} {:?}",pc,num),
            

            // Not needed
            ActionOperation::ScriptSize((_num,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadView((_num,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadViewV((_var,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadPic((_var,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadLogic((_num,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadLogicV((_var,)) => {/* NO-OP-RAGI */},
            ActionOperation::LoadSound((_num,)) => {/* NO-OP-RAGI */},
            ActionOperation::DiscardPic((_var,)) => {/* NO-OP-RAGI */},
            ActionOperation::DiscardView((_num,)) => {/* NO-OP-RAGI */},

            // Everything else
            ActionOperation::If((condition,goto_if_false)) => {
                let mut need_tick=false;
                let new_pc:LogicExecutionPosition;
                if !Self::evaluate_condition(resources,state,condition,&mut need_tick) 
                {
                    new_pc = pc.jump(self,goto_if_false);
                } else {
                    new_pc = pc.next();
                }
                if need_tick {
                    return Some(new_pc.user_input());
                } else {
                    return Some(new_pc);
                }
            },
            ActionOperation::Goto((goto,)) => return Some(pc.jump(self, goto)),
            ActionOperation::Return(()) => return None,
            ActionOperation::Call((num,)) => { let logic = state.get_num(num); return Some(LogicExecutionPosition {logic_file:logic as usize, program_counter: state.get_logic_start(logic), user_input_request: false}) },
            ActionOperation::CallV((var,)) => { let logic = state.get_var(var); return Some(LogicExecutionPosition {logic_file:logic as usize, program_counter: state.get_logic_start(logic), user_input_request: false}) },
            ActionOperation::AssignN((var,num)) => state.set_var(var,state.get_num(num)),
            ActionOperation::AssignV((var1,var2)) => state.set_var(var1,state.get_var(var2)),
            ActionOperation::NewRoom((num,)) => { state.set_new_room(state.get_num(num)); return None },
            ActionOperation::NewRoomV((var,)) => { state.set_new_room(state.get_var(var)); return None },
            ActionOperation::Reset((flag,)) => state.set_flag(flag, false),
            ActionOperation::ResetV((var,)) => { let flag=&TypeFlag::from(state.get_var(var)); state.set_flag(flag, false); },
            ActionOperation::AnimateObj((obj,)) => state.mut_object(obj).set_active(true),
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
            ActionOperation::Reposition((obj,var1,var2)) => {let dx=state.get_var(var1); let dy=state.get_var(var2); state.mut_object(obj).adjust_x_via_delta(dx); state.mut_object(obj).adjust_y_via_delta(dy); },
            ActionOperation::SetPriority((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_priority(n); },
            ActionOperation::SetLoop((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_loop(n); },
            ActionOperation::SetCel((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_cel(n); },
            ActionOperation::DrawPic((var,)) => { let n = state.get_var(var); resources.pictures[&usize::from(n)].render_to(&mut state.video_buffer,&mut state.priority_buffer).unwrap(); },
            ActionOperation::ShowPic(()) => {
                let dpic = double_pic_width(state.picture());
                for y in 0usize..PIC_HEIGHT_USIZE {
                    for x in 0usize..PIC_WIDTH_USIZE*2 {
                        state.back_buffer[x+y*SCREEN_WIDTH_USIZE] = dpic[x+y*SCREEN_WIDTH_USIZE];
                    }
                }
            },
            ActionOperation::ClearLines((num1,num2,num3)) => {
                let start=usize::from(state.get_num(num1) * 8);
                let end = usize::from(state.get_num(num2) * 8)+7;
                let col = state.get_num(num3);
                for y in start..=end {
                    for x in 0usize..SCREEN_WIDTH_USIZE {
                        state.back_buffer[x+y*SCREEN_WIDTH_USIZE] = col;
                    }
                }
            },
            ActionOperation::SetString((s,m)) => { let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); state.set_string(s,m.as_str()); },
            ActionOperation::Draw((obj,)) => state.mut_object(obj).set_visible(true),
            ActionOperation::EndOfLoop((obj,flag)) => { state.set_flag(flag,false); state.mut_object(obj).set_one_shot(flag); },
            ActionOperation::MoveObj((obj,num1,num2,num3,flag)) => { 
                let x=state.get_num(num1); let y=state.get_num(num2); let s=state.get_num(num3); 
                state.set_flag(flag, false);
                state.mut_object(obj).set_move(x, y, s, flag);
                state.mut_object(obj).set_moved(true);
                if *obj==OBJECT_EGO {
                    state.set_program_control();
                }
            },
            ActionOperation::Erase((obj,)) => state.mut_object(obj).set_visible(false),
            ActionOperation::Display((num1,num2,m)) => { let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); let x=state.get_num(num2); let y=state.get_num(num1); Self::display_text(resources,state,x,y,&m,state.get_ink(),state.get_paper()); },
            ActionOperation::DisplayV((var1,var2,var3)) => { let m = self.decode_message_from_resource(state, resources, pc.logic_file, &TypeMessage::from(state.get_var(var3))); let x=state.get_var(var2); let y=state.get_var(var1); Self::display_text(resources,state,x,y,&m,state.get_ink(),state.get_paper()); },
            ActionOperation::ReverseLoop((obj,flag)) => { state.set_flag(flag, false); state.mut_object(obj).set_one_shot_reverse(flag); },
            ActionOperation::Random((num1,num2,var)) => { let r = state.get_random(num1,num2); state.set_var(var,r); },
            ActionOperation::Set((flag,)) => state.set_flag(flag, true),
            ActionOperation::SetV((var,)) => { let flag=&TypeFlag::from(state.get_var(var)); state.set_flag(flag, true); },
            ActionOperation::TextScreen(()) => state.set_text_mode(true),
            ActionOperation::GetString((s,m,num1,num2,num3)) => {
                // This actually halts interpretter until the input string is entered
                let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); 
                let x=state.get_num(num2); 
                let y=state.get_num(num1); 
                let max_length = state.get_num(num3) as usize;
                let input = state.get_string(s).trim_start().to_string().clone();
                let (done,new_string) = command_input(state, input, max_length, &m, resources, x, y,state.get_ink(),state.get_paper(),false);

                *state.get_mut_string(s)=new_string;
                if !done {
                    return Some(pc.user_input());
                }

            },
            ActionOperation::GetNum((m,var)) => {
                // This actually halts interpretter until the input string is entered
                let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); 
                let x=0;
                let y=22;
                let max_length = 5;
                let input = state.get_num_string().clone();
                let (done,new_string) = command_input(state, input, max_length, &m, resources, x, y,state.get_ink(),state.get_paper(),true);

                *state.mut_num_string()=new_string;
                if !done {
                    return Some(pc.user_input());
                } else {
                    // string to num
                    let val = match state.get_num_string().parse::<u8>() {
                        Ok(i) => i,
                        Err(_) => 255,
                    };
                    state.set_var(var,val);
                    state.mut_num_string().clear();
                }

            },
            ActionOperation::Parse((s,)) => parse_input_string(state, state.get_string(s).clone(), resources),
            ActionOperation::SetCursorChar((m,)) => { let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); state.set_prompt(&m); },
            ActionOperation::IgnoreObjs((obj,)) => state.mut_object(obj).set_observing(false),
            ActionOperation::IgnoreBlocks((obj,)) => state.mut_object(obj).set_ignore_barriers(true),
            ActionOperation::StepSize((obj,var)) => {let s=state.get_var(var); state.mut_object(obj).set_step_size(s); },
            ActionOperation::IgnoreHorizon((obj,)) => state.mut_object(obj).set_ignore_horizon(true),
            ActionOperation::StopUpdate((obj,)) => state.mut_object(obj).set_frozen(true),
            ActionOperation::ProgramControl(()) => state.set_program_control(),
            ActionOperation::ObserveBlocks((obj,)) => state.mut_object(obj).set_ignore_barriers(false),
            ActionOperation::Graphics(()) => state.set_text_mode(false),
            ActionOperation::StatusLineOn(()) => state.set_status_visible(true),
            ActionOperation::AcceptInput(()) => state.set_input(true),
            ActionOperation::StartCycling((obj,)) => state.mut_object(obj).set_cycling(true),
            ActionOperation::ObjectOnWater((obj,)) => state.mut_object(obj).set_restrict_to_water(),
            ActionOperation::Wander((obj,)) => state.mut_object(obj).set_wander(),
            ActionOperation::StartUpdate((obj,)) => state.mut_object(obj).set_frozen(false),
            ActionOperation::Distance((obj1,obj2,var)) => state.set_var(var,state.object(obj1).distance(state.object(obj2))),
            ActionOperation::ReleasePriority((obj,)) => { state.mut_object(obj).set_priority_auto(); },
            ActionOperation::PlayerControl(()) => state.set_player_control(),
            ActionOperation::LastCel((obj,var)) => { let cels = get_cells_clamped(resources,state.object(obj)); state.set_var(var,(cels.len()-1) as u8); },
            ActionOperation::SetCelV((obj,var)) => { let n=state.get_var(var); state.mut_object(obj).set_cel(n); },
            ActionOperation::StopMotion((obj,)) => state.mut_object(obj).set_enable_motion(false),
            ActionOperation::NormalMotion((obj,)) => state.mut_object(obj).set_normal_motion(),
            ActionOperation::StartMotion((obj,)) => state.mut_object(obj).set_enable_motion(true),
            ActionOperation::AddN((var,num)) => state.set_var(var,state.get_var(var).wrapping_add(state.get_num(num))),
            ActionOperation::AddV((var1,var2)) => state.set_var(var1,state.get_var(var1).wrapping_add(state.get_var(var2))),
            ActionOperation::SubN((var,num)) => state.set_var(var,state.get_var(var).wrapping_sub(state.get_num(num))),
            ActionOperation::SubV((var1,var2)) => state.set_var(var1,state.get_var(var1).wrapping_sub(state.get_var(var2))),
            ActionOperation::MoveObjV((obj,var1,var2,var3,flag)) => { 
                let x=state.get_var(var1); let y=state.get_var(var2); let s=state.get_var(var3); 
                state.mut_object(obj).set_move(x, y, s, flag);
                if *obj==OBJECT_EGO {
                    state.set_program_control();
                }
            },
            // TODO investigate, Position and RepositionTo act the same, should they
            ActionOperation::Position((obj,num1,num2)) => { let x=state.get_num(num1); let y=state.get_num(num2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y); },
            ActionOperation::RepositionTo((obj,num1,num2)) => { let x=state.get_num(num1); let y=state.get_num(num2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y); },
            ActionOperation::RepositionToV((obj,var1,var2)) => {let x=state.get_var(var1); let y=state.get_var(var2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y); },
            ActionOperation::PositionV((obj,var1,var2)) => { let x=state.get_var(var1); let y=state.get_var(var2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y); },
            ActionOperation::SetTextAttribute((num1,num2)) => { let ink=state.get_num(num1); let paper=state.get_num(num2); state.set_ink(ink); state.set_paper(paper); }
            ActionOperation::StatusLineOff(()) => state.set_status_visible(false),
            ActionOperation::StepTime((obj,var)) => { let time=state.get_var(var); state.mut_object(obj).set_step_time(time); },
            ActionOperation::CycleTime((obj,var)) => { let time=state.get_var(var); state.mut_object(obj).set_cycle_time(time); },
            ActionOperation::ObserveHorizon((obj,)) => state.mut_object(obj).set_ignore_horizon(false),
            ActionOperation::CurrentCel((obj,var)) => { let cur = state.object(obj).get_cel(); state.set_var(var,cur); },
            ActionOperation::CurrentLoop((obj,var)) => { let cur = state.object(obj).get_loop(); state.set_var(var,cur); },
            ActionOperation::CurrentView((obj,var)) => { let cur = state.object(obj).get_view(); state.set_var(var,cur); },
            ActionOperation::FixLoop((obj,)) => state.mut_object(obj).set_fixed_loop(true),
            ActionOperation::AddToPic((num1,num2,num3,num4,num5,num6,num7)) => /* TODO RAGI */ {
                let view=state.get_num(num1);
                let cloop=state.get_num(num2);
                let cel=state.get_num(num3);
                let x=state.get_num(num4) as usize;
                let y=state.get_num(num5) as usize;
                let rpri=state.get_num(num6);
                let margin=state.get_num(num7);
                render_view_to_pic(resources, state, view, cloop, cel, x, y, rpri, margin);
            },
            ActionOperation::SetScanStart(()) => state.set_logic_start(&pc.next()),
            ActionOperation::ResetScanStart(()) => state.clear_logic_start(&pc.next()),
            ActionOperation::FollowEgo((obj,s,f)) => {
                let s=state.get_num(s); 
                state.set_flag(f, false);
                state.mut_object(obj).set_follow(s, f);
            },
            ActionOperation::Toggle((f,)) => { let b=state.get_flag(f); state.set_flag(f, !b); },
            ActionOperation::Get((i,)) => state.set_item_location(i,255),
            ActionOperation::GetV((v,)) => { let i = TypeItem::from(state.get_var(v)); state.set_item_location(&i,255); },
            ActionOperation::Drop((i,)) => state.set_item_location(i,0),
            ActionOperation::Print((m,)) => { 
                let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); 
                if state.displayed != m {
                    state.displayed = m.clone();
                    Self::close_windows(resources, state);
                } 
                if !Self::is_window_open(state) {
                    Self::display_window(resources, state, m.as_str());
                    if !state.get_flag(&FLAG_LEAVE_WINDOW_OPEN) {
                        return Some(pc.user_input());
                    }
                } else {
                    // todo check dialog flag timer thing, for now, any key to exit
                    if !state.get_flag(&FLAG_LEAVE_WINDOW_OPEN) {
                        let key_pressed = state.key_len>0;
                        state.clear_keys();
                        if key_pressed {
                            Self::close_windows(resources, state);
                        } else {
                            return Some(pc.user_input());
                        }
                    }
                }
            },
            ActionOperation::PrintV((var,)) => { 
                let m=&TypeMessage::from(state.get_var(var)); 
                let m = self.decode_message_from_resource(state, resources, pc.logic_file, m); 
                if state.displayed != m {
                    state.displayed = m.clone();
                    Self::close_windows(resources, state);
                } 
                if !Self::is_window_open(state) {
                    Self::display_window(resources, state, m.as_str());
                    if !state.get_flag(&FLAG_LEAVE_WINDOW_OPEN) {
                        return Some(pc.user_input());
                    }
                } else {
                    // todo check dialog flag timer thing, for now, any key to exit
                    if !state.get_flag(&FLAG_LEAVE_WINDOW_OPEN) {
                        let key_pressed = state.key_len>0;
                        state.clear_keys();
                        if key_pressed {
                            Self::close_windows(resources, state);
                        } else {
                            return Some(pc.user_input());
                        }
                    }
                }
            },
            ActionOperation::ShowObj((num,)) => {
                let v = state.get_num(num) as usize;
                let view = &resources.views[&v];
                let m = view.get_description();
                if state.displayed != *m {
                    state.displayed = m.clone();
                    Self::close_windows(resources, state);
                } 
                if !Self::is_window_open(state) {
                    Self::display_window(resources, state, m.as_str());
                    Self::display_obj(resources, state, view);

                    return Some(pc.user_input());
                } else {
                    // todo check dialog flag timer thing, for now, any key to exit
                    let key_pressed = state.key_len>0;
                    state.clear_keys();
                    if key_pressed {
                        Self::close_windows(resources, state);
                    } else {
                        return Some(pc.user_input());
                    }
                }
            },
            ActionOperation::CloseWindow(()) => {
                Self::close_windows(resources, state);
                state.set_flag(&FLAG_LEAVE_WINDOW_OPEN, false);
            }


            _ => panic!("TODO {:?}:{:?}",pc,action),
        }

        Some(pc.next())
    }
 
    pub fn render_glyph(resources:&GameResources,state:&mut LogicState,x:u16,y:u8,g:u8,ink:u8,paper:u8) {
        let s = resources.font.as_slice();
        let x = x as usize;
        let y = y as usize;
        for yy in 0..8 {
            let index = (g as usize)*8 + 4 + yy;
            let mut bits = s[index];
            for xx in 0..8 {
                if (bits & 0x80) == 0x80 {
                    state.text_buffer[x+xx+(y+yy)*SCREEN_WIDTH_USIZE] = ink;
                } else {
                    state.text_buffer[x+xx+(y+yy)*SCREEN_WIDTH_USIZE] = paper;
                }
                bits<<=1;
            }
        }
    }

    pub fn display_text(resources:&GameResources,state:&mut LogicState,x:u8,y:u8,s:&String,ink:u8,paper:u8) {
        let mut x = (x as u16)*8;
        let y=y*8;
        for l in s.as_bytes() {
            Self::render_glyph(resources, state, x, y, *l,ink,paper);
            x+=8;
        }
    }

    pub fn open_window(resources:&GameResources,state:&mut LogicState,w:usize,x0:u16,y0:u8,x1:u16,y1:u8,ink:u8,paper:u8) {

        state.windows[w].x0=x0;
        state.windows[w].x1=x1;
        state.windows[w].y0=y0;
        state.windows[w].y1=y1;

        Self::render_glyph(resources, state, x0*8, y0*8, 218 , ink, paper);
        Self::render_glyph(resources, state, x1*8, y0*8, 191 , ink, paper);
        Self::render_glyph(resources, state, x0*8, y1*8, 192 , ink, paper);
        Self::render_glyph(resources, state, x1*8, y1*8, 217 , ink, paper);
        for x in (x0+1)..x1 {
            Self::render_glyph(resources, state, x*8, y0*8, 196 , ink, paper);
            Self::render_glyph(resources, state, x*8, y1*8, 196 , ink, paper);
        }
        for y in (y0+1)..y1 {
            Self::render_glyph(resources, state, x0*8, y*8, 179 , ink, paper);
            Self::render_glyph(resources, state, x1*8, y*8, 179 , ink, paper);
            for x in (x0+1)..x1 {
                Self::render_glyph(resources, state, x*8, y*8, 32 , ink, paper);
            }
        }
    }

    pub fn is_window_open(state:&LogicState) -> bool {
        !state.windows[0].is_empty() || !state.windows[1].is_empty()
    }

    pub fn close_windows(resources:&GameResources,state:&mut LogicState) {
        for w in 0..state.windows.len() {
            if !state.windows[w].is_empty() {
                for y in state.windows[w].y0..=state.windows[w].y1 {
                    for x in state.windows[w].x0..=state.windows[w].x1 {
                        Self::render_glyph(resources, state, x*8, y*8, 32 , 4, 255);
                    }
                }
            }
        }
        state.windows[0] = TextWindow::new();
        state.windows[1] = TextWindow::new();
    }

    pub fn display_obj(resources:&GameResources,state:&mut LogicState, view:&ViewResource) {
        let view_loop=&view.get_loops()[0];
        let view_cel=&view_loop.get_cels()[0];

        let width=(view_cel.get_width() as u16)*2;
        let height=view_cel.get_height();
        let char_width=(width+7)/8;
        let char_height=(height+7)/8;

        let y1=21;  // (bottom of view 168)
        let y0=y1-char_height;
        let x0=20-char_width/2;
        let x1=x0+char_width;

        Self::open_window(resources,state,1,x0,y0,x1,y1,255,255);

        render_view_to_window(view_cel, state, (x0*8+8).into(), (y1*8).into());
    }

    pub fn display_window(resources:&GameResources,state:&mut LogicState, message:&str) {

        // compute window size
        let mut max_width=0;
        let mut width=0u16;
        let mut word_len=0;
        let mut height=1u8;
        let mut iter = 0;
        let bytes = message.as_bytes();
        let mut splits=[999usize;40];

        while iter < bytes.len() {
            let c=bytes[iter];
            if width>=30 {
                iter-=word_len as usize;
                splits[(height-1) as usize]=iter;
                height+=1;
                width-=word_len;
                max_width=if max_width < width { width } else {max_width};
                width=0;
                word_len=0;
            } else {
                if c==b'\n' {
                    splits[(height-1)as usize]=iter;
                    height+=1;
                    max_width=if max_width < width { width } else {max_width};
                    width=0;
                    word_len=0;
                } else if c==b' ' {
                    width+=1;
                    word_len=0;
                } else {
                    width+=1;
                    word_len+=1;
                }
                iter+=1;
            }
        }
        max_width=if max_width < width { width } else {max_width};

        height+=1;
        max_width+=1;

        let y0 = 10 - height/2;
        let x0 = 20 - max_width/2;
        let y1 = y0 + height;
        let x1 = x0 + max_width;

        Self::open_window(resources,state,0,x0,y0,x1,y1,4,15);

        let mut x=(x0+1)*8;
        let mut y=(y0+1)*8;
        let mut split_loc=0;
        for (idx,c) in bytes.iter().enumerate() {
            if splits[split_loc]==idx {
                x=(x0+1)*8;
                y+=8;
                split_loc+=1;
            }
            if *c!=b'\n' {
                Self::render_glyph(resources, state, x, y, *c, 0, 15);
                x+=8;
            }
        }
    }

    pub fn interpret_instructions(&self,resources:&GameResources,state:&mut LogicState,pc:&LogicExecutionPosition,actions:&[LogicOperation]) -> Option<LogicExecutionPosition> {
        self.interpret_instruction(resources, state, pc, &actions[pc.program_counter].action)
    }

}

pub fn parse_input_string(state: &mut LogicState, s: String, resources: &GameResources) {
    state.parsed_input_string = s.trim().to_ascii_lowercase();
    let mut w_idx=0usize;
    state.words=[0u16;256];
    state.set_var(&VAR_MISSING_WORD,0);
    for (index,w) in state.parsed_input_string.split(' ').enumerate() {
        let t = w.trim();
        if !t.is_empty() {
            match resources.words.get(t) {
                None => { state.set_var(&VAR_MISSING_WORD, index.saturating_add(1) as u8); break; },
                Some(0u16) => {},
                Some(b) => { state.words[w_idx]=*b; w_idx+=1; },
            }
        }
    }
    state.set_flag(&FLAG_COMMAND_ENTERED,state.parsed_input_string.len()!=0);
    state.set_flag(&FLAG_SAID_ACCEPTED_INPUT,false);
}

pub fn command_input(state: &mut LogicState, s: String, max_length: usize, m: &String, resources: &GameResources, x: u8, y: u8, ink:u8, paper:u8, number_only:bool) -> (bool,String) {
    let mut new_string = s;
    let mut done = false;
    for a in 0..state.key_len {
        let c = state.key_buffer[a];
        match c {
            13 => { done=true; break; },
            8 => { new_string.pop(); },
            b'a'..=b'z' => if !number_only && new_string.len() < max_length {new_string.push(char::from(c)) },
            b'A'..=b'Z' => if !number_only && new_string.len() < max_length {new_string.push(char::from(c)) },
            b'0'..=b'9' => if new_string.len() < max_length {new_string.push(char::from(c)) },
            32 => if !number_only && new_string.len() < max_length {new_string.push(char::from(c)) },
            _ => {},
        }
    }
    state.clear_keys();
    
    // Go through keyboard buffer and append/remove keys?
    let to_show = m.clone()+new_string.as_str()+state.get_prompt().to_string().as_str();
    let indent_len = if max_length+1<new_string.len() { 0 } else {(max_length+1) - new_string.len()};
    let to_show = to_show + format!("{:indent$}","",indent=indent_len).as_str();
    LogicSequence::display_text(resources, state, x, y, &to_show,ink,paper);
    // pull keycodes off 
    (done,new_string)
}

#[derive(Copy,Clone,Debug,Hash,PartialEq,Eq)]
pub struct LogicExecutionPosition {
    logic_file:usize,
    program_counter:usize,
    user_input_request:bool,
}

impl LogicExecutionPosition {
    pub fn new(file:usize,pc:usize) -> LogicExecutionPosition {
        LogicExecutionPosition { logic_file: file, program_counter: pc, user_input_request: false }
    }

    pub fn user_input(&self) -> LogicExecutionPosition {
        // will cause the interpretter to stop and return back to this location after a render tick
        LogicExecutionPosition { logic_file: self.logic_file, program_counter: self.program_counter, user_input_request: true }
    }

    pub fn next(&self) -> LogicExecutionPosition {
        LogicExecutionPosition { logic_file: self.logic_file, program_counter: self.program_counter+1, user_input_request: false }
    }

    pub fn jump(&self, sequence:&LogicSequence, goto:&TypeGoto) -> LogicExecutionPosition {
        LogicExecutionPosition { logic_file: self.logic_file, program_counter: sequence.lookup_offset(goto).unwrap(), user_input_request: false }
    }

    pub fn is_call(&self,logic_file:usize) -> bool {
        self.logic_file!=logic_file
    }

    pub fn is_input_request(&self) -> bool {
        self.user_input_request
    }

    pub fn get_logic(&self) -> usize {
        self.logic_file
    }
    
    pub fn get_pc(&self) -> usize {
        self.program_counter
    }

}

impl fmt::Display for LogicExecutionPosition {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f,"({:3}:{:3})",self.logic_file,self.program_counter)
    }
}

pub fn get_direction_from_delta(dx:i32,dy:i32) -> u8 {

    match (dx,dy) {
        (-1,-1) => 8,
        (-1, 0) => 7,
        (-1, 1) => 6,
        ( 0,-1) => 1,
        ( 0, 0) => 0,
        ( 0, 1) => 5,
        ( 1,-1) => 2,
        ( 1, 0) => 3,
        ( 1, 1) => 4,
        _ => panic!("get_direction_from_delta called with non signum values {},{}",dx,dy),
    }
}

pub fn get_delta_from_direction(direction:u8) -> (i8,i8) {
    match direction {
        0 => ( 0, 0),
        1 => ( 0,-1),
        2 => ( 1,-1),
        3 => ( 1, 0),
        4 => ( 1, 1),
        5 => ( 0, 1),
        6 => (-1, 1),
        7 => (-1, 0),
        8 => (-1,-1),
        _ => panic!("get_delta_fp32_from_direction called with invalid direction {}",direction),
    }

}
pub fn get_delta_fp32_from_direction(direction:u8) -> (FP32,FP32) {
    let (x,y) = get_delta_from_direction(direction);
    (FP32::from(x),FP32::from(y))
}

//sprite stuff
pub fn update_sprites(resources:&GameResources,state:&mut LogicState) {
    // Handle direction updates/move logic?

    for num in state.active_objects_indices() {
        let obj_num = &TypeObject::from(num as u8);
        if !(state.object(obj_num).visible && !state.object(obj_num).frozen) {
            continue;
        }

        if !state.mut_object(obj_num).should_step() {
            continue;
        }

        match state.object(obj_num).get_motion_kind() {
            SpriteMotion::Normal => {}, // what ever is in direction is used
            SpriteMotion::Wander => {   // update direction randomly, if didn't move last time
                if !state.object(obj_num).has_moved() {
                    let direction = state.rng.gen_range(0u8..=8);
                    state.mut_object(obj_num).set_direction(direction);
                }
            },
            SpriteMotion::MoveObj => {
                let x=FP32::from(state.object(obj_num).get_x_fp16());
                let y=FP32::from(state.object(obj_num).get_y_fp16());
                let ex=FP32::from(state.object(obj_num).get_end_x());
                let ey=FP32::from(state.object(obj_num).get_end_y());
                let dx = (ex.int()-x.int()).signum();
                let dy = (ey.int()-y.int()).signum();
                let direction = get_direction_from_delta(dx.to_num(), dy.to_num());
                state.mut_object(obj_num).set_direction(direction);
                if direction==0 || !state.object(obj_num).has_moved() {
                    let mflag = state.object(obj_num).move_flag;
                    state.set_flag(&mflag, true);
                    state.mut_object(obj_num).clear_move();
                }
            },
            SpriteMotion::FollowEgo => {
                let x=FP32::from(state.object(obj_num).get_x_fp16());
                let y=FP32::from(state.object(obj_num).get_y_fp16());
                let ex=FP32::from(state.object(&OBJECT_EGO).get_x_fp16());
                let ey=FP32::from(state.object(&OBJECT_EGO).get_y_fp16());
                let dx = (ex.int()-x.int()).signum();
                let dy = (ey.int()-y.int()).signum();
                let direction = get_direction_from_delta(dx.to_num(), dy.to_num());
                state.mut_object(obj_num).set_direction(direction);
                if direction==0 {
                    let mflag = state.object(obj_num).move_flag;
                    state.set_flag(&mflag, true);
                    state.mut_object(obj_num).clear_move();
                }
            },
        }

        if state.object(obj_num).get_direction()!=0 {
            // Now perform motion based on direction
            // Collision/rules check here I think
            let (moved, nx,ny,water,signal) = update_move(resources,state,obj_num);
            if *obj_num == OBJECT_EGO {
                state.set_flag(&FLAG_EGO_TOUCHED_SIGNAL, signal);
                state.set_flag(&FLAG_EGO_IN_WATER,water);
            }


            state.mut_object(obj_num).set_moved(moved);
            if moved {
                state.mut_object(obj_num).set_x_fp16(nx);
                state.mut_object(obj_num).set_y_fp16(ny);
            }
        } else {
            state.mut_object(obj_num).set_moved(false);
        }

    }
}

fn update_edge(state:&mut LogicState,obj_num:&TypeObject,edge:u8) {
    if *obj_num == OBJECT_EGO {
        state.set_var(&VAR_EGO_EDGE,edge);
    } else {
        state.set_var(&VAR_OBJ_EDGE,edge);
        state.set_var(&VAR_OBJ_TOUCHED_BORDER,obj_num.value);
    }
}

pub fn update_move(resources:&GameResources,state:&mut LogicState,obj_num:&TypeObject) -> (bool,FP16,FP16,bool,bool) {

    let obj=state.object(&obj_num);

    if !obj.motion {
        return (false,FP16::ZERO,FP16::ZERO,false,false);
    }
    let (dx,dy) = get_delta_fp32_from_direction(obj.get_direction());
    let x=FP32::from(obj.get_x_fp16());
    let y=FP32::from(obj.get_y_fp16());
    let s=FP32::from(obj.get_step_size());
    let x=x+dx*s;
    let y=y+dy*s;

    let bx:i32 = x.to_bits();
    let by:i32 = y.to_bits();
    let nx = FP16::from_bits((bx&0xFFFF) as u16);
    let ny = FP16::from_bits((by&0xFFFF) as u16);

    let mut c = usize::from(obj.get_cel());
    let cels = get_cells_clamped(resources, &obj);
    if c>=cels.len() {
        c=cels.len()-1;
    }
    let cell = &cels[c];

    let w = cell.get_width() as usize;
    let _h = cell.get_height() as usize;
    let tx:usize = nx.to_num();
    let ty:usize = ny.to_num();
    
    // clip in screen bounds (should these block or clip?)
    if x<0 || y<0 { // y should ideally check y-h
        if x<0 {
            update_edge(state,obj_num,4);
        } else {
            update_edge(state,obj_num,1);
        }
        return (false,FP16::ZERO,FP16::ZERO,false,false);
    }
    if tx+w > PIC_WIDTH_USIZE || ty >= PIC_HEIGHT_USIZE {
        if tx+w > PIC_WIDTH_USIZE {
            update_edge(state,obj_num,2);
        } else {
            update_edge(state,obj_num,3);
        }

        return (false,FP16::ZERO,FP16::ZERO,false,false);
    }

    // horizon check
    if !obj.ignore_horizon {
        if ty < (state.horizon as usize) {
            update_edge(state,obj_num,1);
            return (false,FP16::ZERO,FP16::ZERO,false,false);
        }
    }
    // todo checks for other block and sprites?

    // scan x+0..x+width-1 and confirm priority as expected
    let mut blocked=false;
    let mut water=true;
    let mut signal=false;
    for x in 0..w {
        let pri = fetch_priority_for_pixel(state, tx+x, ty);
        water&=pri==3;
        if pri == 3 && obj.is_restricted_to_land() {
            blocked=true;
        }
        if pri != 3 && obj.is_restricted_to_water() {
            blocked=true;
        }
        if pri == 0 {
            blocked=true;
        }
        if pri == 1 {
            if obj.is_restricted_by_blocks() {
                blocked=true;
            }
        }
        if pri == 2 {
            signal=true;
        }
    }

    if blocked {
        return (false,FP16::ZERO,FP16::ZERO,water,signal);
    }
    (true, nx, ny, water, signal)

}

pub fn fetch_priority_for_pixel(state:&LogicState,x:usize,y:usize) -> u8 {
    state.priority()[x+y*PIC_WIDTH_USIZE]
}

pub fn fetch_priority_for_pixel_rendering(state:&LogicState,x:usize,y:usize) -> u8 {
    let mut pri:u8 = 0;
    let mut y = y;
    while y<168 && pri<3 {
        pri = fetch_priority_for_pixel(state, x, y);
        y+=1;
    }
    if pri<3 {
        return 15;  // bottom of screen
    }
    pri
}
//
// x1,y1
//  |
//  +---- x2,y2
pub fn is_left_edge_in_box(_resources:&GameResources,state:&LogicState,obj:&TypeObject,x1:&TypeNum,y1:&TypeNum,x2:&TypeNum,y2:&TypeNum) -> bool {
    let obj = state.object(obj);
    let x1=state.get_num(x1);
    let y1=state.get_num(y1);
    let x2=state.get_num(x2);
    let y2=state.get_num(y2);
    let x=obj.get_x();
    let y=obj.get_y();
    x>=x1 && x<=x2 && y>=y1 && y<=y2
}

pub fn is_center_edge_in_box(resources:&GameResources,state:&LogicState,obj:&TypeObject,x1:&TypeNum,y1:&TypeNum,x2:&TypeNum,y2:&TypeNum) -> bool {
    let obj = state.object(obj);
    let c = usize::from(obj.get_cel());
    let cels = get_cells_clamped(resources, obj);
    let cell = &cels[c];
    let x1=state.get_num(x1);
    let y1=state.get_num(y1);
    let x2=state.get_num(x2);
    let y2=state.get_num(y2);
    let x=obj.get_x() + cell.get_width()/2;
    let y=obj.get_y();
    x>=x1 && x<=x2 && y>=y1 && y<=y2
}

pub fn is_right_edge_in_box(resources:&GameResources,state:&LogicState,obj:&TypeObject,x1:&TypeNum,y1:&TypeNum,x2:&TypeNum,y2:&TypeNum) -> bool {
    let obj = state.object(obj);
    let c = usize::from(obj.get_cel());
    let cels = get_cells_clamped(resources, obj);
    let cell = &cels[c];
    let x1=state.get_num(x1);
    let y1=state.get_num(y1);
    let x2=state.get_num(x2);
    let y2=state.get_num(y2);
    let x=obj.get_x() + cell.get_width()-1;
    let y=obj.get_y();
    x>=x1 && x<=x2 && y>=y1 && y<=y2
}

pub fn is_left_and_right_edge_in_box(resources:&GameResources,state:&LogicState,obj:&TypeObject,x1:&TypeNum,y1:&TypeNum,x2:&TypeNum,y2:&TypeNum) -> bool {
    is_left_edge_in_box(resources,state,obj,x1,y1,x2,y2) && is_right_edge_in_box(resources,state,obj,x1,y1,x2,y2)
}

// Todo cache these in the sprites
pub fn get_loops<'a>(resources:&'a GameResources,obj:&Sprite) -> &'a Vec<ViewLoop> {
    let v = usize::from(obj.get_view());
    let view = &resources.views[&v];
    view.get_loops()
}

pub fn get_cells_clamped<'a>(resources:&'a GameResources,obj:&Sprite) -> &'a Vec<ViewCel> {
    let v = usize::from(obj.get_view());
    let mut l = usize::from(obj.get_loop());
    let view = &resources.views[&v];
    let loops = view.get_loops();
    if l>=loops.len() { 
        l=loops.len()-1;
    }
    let cloop = &loops[l];
    cloop.get_cels()
}


pub fn update_anims(resources:&GameResources,state:&mut LogicState) {
    for num in state.active_objects_indices() {
        let obj_num = TypeObject::from(num as u8);
        let c = usize::from(state.object(&obj_num).get_cel());

        if !state.object(&obj_num).frozen {
            if !state.object(&obj_num).fixed_loop {
                let loops = get_loops(resources, state.object(&obj_num));
                match loops.len() {
                    0..=1 => {},    // Do nothing
                    2..=3 => {
                        let direction = state.object(&obj_num).get_direction();
                        match direction {
                            0..=1 | 5 => {}, // Do nothing
                            2..=4 => state.mut_object(&obj_num).set_loop(0),
                            6..=8 => state.mut_object(&obj_num).set_loop(1),
                            _ => panic!("direction not valid range for auto loop {}",direction),
                        }
                    },
                    4..=7 => {
                        let direction = state.object(&obj_num).get_direction();
                        match direction {
                            0 => {}, // Do nothing
                            1 => state.mut_object(&obj_num).set_loop(3),
                            2..=4 => state.mut_object(&obj_num).set_loop(0),
                            5 => state.mut_object(&obj_num).set_loop(2),
                            6..=8 => state.mut_object(&obj_num).set_loop(1),
                            _ => panic!("direction not valid range for auto loop {}",direction),
                        }
                    },
                    _ => panic!("Unsupported loop count in auto loop {}",loops.len()),
                }
            }

            // update cells in case we have switched loop
            let cels = get_cells_clamped(resources, state.object(&obj_num));

            if state.object(&obj_num).cycle {
                if !state.mut_object(&obj_num).should_cycle() {
                    continue;
                }
                let ccel = state.object(&obj_num).get_cel();
                let last_cel = cels.len()-1;
                // Next cel if able
                match state.object(&obj_num).cycle_kind {
                    SpriteCycle::Reverse => {
                        if c > 0 {
                            state.mut_object(&obj_num).set_cel(ccel.wrapping_sub(1));
                        } else {
                            state.mut_object(&obj_num).set_cel(last_cel as u8);
                        }
                    },
                    SpriteCycle::OneShotReverse => {
                        if c > 0 {
                            state.mut_object(&obj_num).set_cel(ccel.wrapping_sub(1));
                        } else {
                            let oflag = state.object(&obj_num).cycle_flag;
                            state.set_flag(&oflag,true);
                            state.mut_object(&obj_num).end_one_shot();
                        }
                    },
                    SpriteCycle::Normal => {
                        if last_cel > c {
                            state.mut_object(&obj_num).set_cel(ccel.wrapping_add(1));
                        } else {
                            state.mut_object(&obj_num).set_cel(0);
                        }
                    }
                    SpriteCycle::OneShot => {
                        if last_cel > c {
                            state.mut_object(&obj_num).set_cel(ccel.wrapping_add(1));
                        } else {
                            let oflag = state.object(&obj_num).cycle_flag;
                            state.set_flag(&oflag,true);
                            state.mut_object(&obj_num).end_one_shot();
                        }
                    }
                }
            }
        }
    }

}

pub fn render_sprites(resources:&GameResources,state:&mut LogicState, disable_background:bool) {
    state.post_sprites = if disable_background {[0u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE]} else {state.back_buffer};

    for num in state.active_objects_indices_sorted_y() {
        let obj_num = TypeObject::from(num as u8);
        let mut c = usize::from(state.object(&obj_num).get_cel());
        let cels = get_cells_clamped(resources, state.object(&obj_num));
        if c>=cels.len() {
            c=cels.len()-1;
        }
        let cell = &cels[c];

        if state.object(&obj_num).visible {
            render_sprite(&obj_num, cell, state);
        }
    }
}

fn render_view_to_pic(resources: &GameResources, state:&mut LogicState, view:u8, cloop:u8, cel:u8, x:usize, y:usize, rpri:u8, margin:u8) {
    let view = &resources.views[&(view as usize)];
    let loops = &view.get_loops()[cloop as usize];
    let cel = &loops.get_cels()[cel as usize];
    let t = cel.get_transparent_colour();
    let d = cel.get_data();
    let mirror=cel.is_mirror(cloop);
    let w = cel.get_width().into();
    let h = cel.get_height().into();
    for yy in 0..h {
        for xx in 0..w {
            let col = if mirror {d[(w-xx-1)+yy*w]} else {d[xx+yy*w] };
            if col != t {
                let sx = xx+x;
                let sy = yy+y-h;
                let coord = sx+sy*PIC_WIDTH_USIZE;
                let pri = fetch_priority_for_pixel_rendering(state,sx,sy);
                if pri <= rpri {
                    state.mut_picture()[coord]=col;
                }
            }
        }
    }
    // render control value
    if margin<4 {
        for xx in 0..w {
            let sx = xx+x;
            let sy = y;
            let coord = sx+sy*PIC_WIDTH_USIZE;
            state.priority_buffer[coord]=margin;
        }
    }
}

fn render_view_to_window(cell: &view::ViewCel, state: &mut LogicState, x:usize, y:usize) {
    let h = usize::from(cell.get_height());
    let w = usize::from(cell.get_width());
    let t = cell.get_transparent_colour();
    let d = cell.get_data();
    for yy in 0..h {
        for xx in 0..w {
            let col = d[xx+yy*w];
            if col != t {
                let sx = xx*2+x;
                let sy = yy+y-h;
                    // We double the pixels of sprites at this point
                let coord = sx+sy*SCREEN_WIDTH_USIZE;
                state.text_buffer[coord]=col;
                state.text_buffer[coord+1]=col;
            }
        }
    }
}

fn render_sprite(obj_num:&TypeObject, cell: &view::ViewCel, state: &mut LogicState) {
    let x = usize::from(state.object(obj_num).get_x());
    let y = usize::from(state.object(obj_num).get_y());
    let mut h = usize::from(cell.get_height());
    let w = usize::from(cell.get_width());
    let t = cell.get_transparent_colour();
    let d = cell.get_data();
    let mirror=cell.is_mirror(state.object(obj_num).get_loop());
    if y<h {
        h=y;
    }
    for yy in 0..h {
        for xx in 0..w {
            let col = if mirror {d[(w-xx-1)+yy*w]} else {d[xx+yy*w] };
            if col != t {
                let sx = xx+x;
                let sy = yy+y-h;
                let pri = fetch_priority_for_pixel_rendering(state,sx,sy);
                if pri <= state.object(obj_num).get_priority() {
                    // We double the pixels of sprites at this point
                    let coord = sx*2+sy*SCREEN_WIDTH_USIZE;
                    state.post_sprites[coord]=col;
                    state.post_sprites[coord+1]=col;
                }
            }
        }
    }
}
