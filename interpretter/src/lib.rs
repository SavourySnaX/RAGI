use std::{collections::HashMap, fs, fmt, cmp::Ordering};

use dir_resource::{Root, ResourceDirectory, ResourceType, ResourcesVersion};
use fixed::{FixedU16, FixedI32, types::extra::U8};
use from_to_repr::FromToRepr;
use helpers::double_pic_width;
use itertools::Itertools;
use logic::*;
use objects::{Objects, Object};
use picture::*;
use rand::{rngs::ThreadRng, Rng, random};
use view::{ViewResource, ViewLoop, ViewCel};
use volume::Volume;
use words::Words;

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

pub const VAR_MESSAGE_WINDOW_TIMER:TypeVar = type_var_from_u8(21);

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
    move_step:FP16,
    wander_distance:FP16,
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
            move_step: FP16::from_num(0),
            wander_distance: FP16::from_num(0),
            ex: FP16::from_num(0),
            ey: FP16::from_num(0),
            step_size: FP16::from_num(1),
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
        !self.ignore_barriers
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
        if self.cycle_time == 0 {
            return false;
        }
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
        self.set_normal_motion();
        self.set_enable_motion(true);
        self.set_cycling(true);
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
        self.step_size=FP16::from_num(s as u16);
    }

    pub fn restore_step_size(&mut self) {
        self.step_size=self.move_step;
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
   
    // Todo frozen should mean view/loop/cel is not changed if object on screen, will need a shadow of these vars to do that (or cache the cell in the sprite, and direct render that - better)
    pub fn set_view(&mut self, view:u8, resources:&GameResources) {
        self.view = view;
        if resources.views[&(view as usize)].get_loops().len()<=(self.cloop as usize) {
            self.cloop=0;
        }
    }

    pub fn set_loop(&mut self,n:u8) {
        self.cloop = n;
    }
    
    pub fn set_cel(&mut self,n:u8) {
        self.cel = n;
        println!("TODO - if object no longer fits, reposition along required edge");
    }

    pub fn set_fixed_loop(&mut self,b:bool) {
        self.fixed_loop = b;
    }

    pub fn set_normal_cycle(&mut self) {
        self.cycle_kind=SpriteCycle::Normal;
        self.cycle=true;
    }

    pub fn set_reverse_cycle(&mut self) {
        self.cycle_kind=SpriteCycle::Reverse;
        self.cycle=true;
    }

    pub fn set_one_shot(&mut self,f:&TypeFlag) {
        self.cycle_kind=SpriteCycle::OneShot;
        self.set_frozen(false);
        self.set_cycling(true);
        self.cycle_flag = *f;
    }

    pub fn set_one_shot_reverse(&mut self,f:&TypeFlag) {
        self.cycle_kind=SpriteCycle::OneShotReverse;
        self.set_frozen(false);
        self.set_cycling(true);
        self.cycle_flag = *f;
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
        self.move_step=self.get_step_size();
        if s!=0 {
            self.set_step_size(s);
        }
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

    pub fn set_wander(&mut self,dist:u8) {
        self.motion_kind=SpriteMotion::Wander;
        self.wander_distance=FP16::from_num(dist);
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

        let root = Root::new(base_path,version);
    
        let mut volumes:HashMap<u8,Volume>=HashMap::new();

        let dir = ResourceDirectory::new(&root,ResourceType::Views).unwrap();

        let mut views:HashMap<usize,ViewResource> = HashMap::new();
        views.reserve(256);
        for (index,entry) in dir.into_iter().enumerate() {
            if !entry.empty() {
                if let std::collections::hash_map::Entry::Vacant(e) =volumes.entry(entry.volume) {
                    let bytes = root.fetch_volume(&entry);
                    e.insert(Volume::new(bytes.into_iter())?);
                }
                views.insert(index, ViewResource::new(&volumes[&entry.volume],&entry)?);
            }
        }
        views.shrink_to_fit();

        let dir = ResourceDirectory::new(&root, ResourceType::Pictures).unwrap();

        let mut pictures:HashMap<usize,PictureResource> = HashMap::new();
        pictures.reserve(256);
        for (index,entry) in dir.into_iter().enumerate() {
            if !entry.empty() {
                if let std::collections::hash_map::Entry::Vacant(e) = volumes.entry(entry.volume) {
                    let bytes = root.fetch_volume(&entry);
                    e.insert(Volume::new(bytes.into_iter())?);
                }
                pictures.insert(index, PictureResource::new(&volumes[&entry.volume],&entry)?);
            }
        }
        pictures.shrink_to_fit();

        let dir = ResourceDirectory::new(&root, ResourceType::Logic).unwrap();

        let mut logic:HashMap<usize,LogicResource> = HashMap::new();
        logic.reserve(256);
        for (index,entry) in dir.into_iter().enumerate() {
            if !entry.empty() {
                if let std::collections::hash_map::Entry::Vacant(e) = volumes.entry(entry.volume) {
                    let bytes = root.fetch_volume(&entry);
                    e.insert(Volume::new(bytes.into_iter())?);
                }
                logic.insert(index, LogicResource::new(&volumes[&entry.volume],&entry,&ResourcesVersion::new(version))?);
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
    controllers:HashMap<u8,AgiKeyCodes>,
    key_len:usize,
    key_buffer:[AgiKeyCodes;256],

    // video
    picture_buffer:[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],
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
            key_buffer:[AgiKeyCodes::Enter;256],
            controllers:HashMap::new(),
            picture_buffer:[15;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],
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
        self.flag[f.get_value() as usize]
    }

    pub fn get_vars(&self) -> impl Iterator<Item = u8> {
        self.var.into_iter()
    }

    pub fn get_var(&self,v:&TypeVar) -> u8 {
        self.var[v.get_value() as usize]
    }

    pub fn get_num(&self,v:&TypeNum) -> u8 {
        v.get_value()
    }

    pub fn get_logic_start(&self,l:u8) -> usize {
        self.logic_start[l as usize]
    }

    pub fn get_controller(&self,c:&TypeController) -> u8 {
        c.get_value()
    }

    pub fn get_new_room(&self) -> u8 {
        self.new_room
    }

    pub fn get_item_room(&self,item:&TypeItem) -> u8 {
        self.item_location[item.get_value() as usize]
    }

    pub fn get_message(&self,m:&TypeMessage) -> u8 {
        m.get_value()
    }

    pub fn get_strings(&self) -> impl Iterator<Item = &String> {
        self.string.iter()
    }

    pub fn get_num_string(&self) -> &String {
        &self.num_string
    }

    pub fn get_string(&self,s:&TypeString) -> &String {
        &self.string[s.get_value() as usize]
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
            if word.get_value() == 1 && self.words[index]==0 {
                return false;
            }
            // Match remainder of input
            if word.get_value() == 9999 {
                break;
            }
            // Word does not match
            if word.get_value() != self.words[index] {
                return false;
            }
        }
        self.set_flag(&FLAG_SAID_ACCEPTED_INPUT, true);
        true
    }
    
    pub fn unanimate_all(&mut self) {
        for (_,obj) in self.mut_active_objects() {
            obj.set_active(false);
        }
    }

    pub fn mut_num_string(&mut self) -> &mut String {
        &mut self.num_string
    }

    pub fn get_mut_string(&mut self,s:&TypeString) -> &mut String {
        &mut self.string[s.get_value() as usize]
    }

    pub fn get_random(&mut self,start:&TypeNum,end:&TypeNum) -> u8 {
        let s=self.get_num(start);
        let e=self.get_num(end);
        if s==e {
            return s;
        }
        self.rng.gen_range(self.get_num(start)..self.get_num(end))
    }

    pub fn set_logic_start(&mut self,pos:&LogicExecutionPosition) {
        self.logic_start[pos.logic_file]=pos.program_counter;
    }

    pub fn clear_logic_start(&mut self,pos:&LogicExecutionPosition) {
        self.logic_start[pos.logic_file]=0;
    }

    pub fn set_var(&mut self,v:&TypeVar,n:u8) {
        self.var[v.get_value() as usize] = n;
    }

    pub fn set_flag(&mut self,f:&TypeFlag,n:bool) {
        self.flag[f.get_value() as usize] = n;
    }

    pub fn set_string(&mut self,s:&TypeString,m:&str) {
        self.string[s.get_value() as usize] = m.to_owned();
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
        self.item_location[item.get_value() as usize]=loc;
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
        &self.objects[o.get_value() as usize]
    }

    pub fn mut_object(&mut self,o:&TypeObject) -> &mut Sprite {
        &mut self.objects[o.get_value() as usize]
    }

    pub fn active_objects_indices(&self) -> impl Iterator<Item = usize>{
        let t_indices:Vec<usize> = (0..self.objects.len())
            .filter(|b| self.object(&type_object_from_u8(*b as u8)).is_active())
            .collect_vec();
        t_indices.into_iter()
    }
    
    fn compare_pri_y(&self,a:&Sprite,b:&Sprite) -> Ordering {
        let ap = a.get_priority();
        let bp = b.get_priority();
        if ap<bp {
            return Ordering::Less;
        } else if ap>bp {
            return Ordering::Greater;
        } else {
            Ord::cmp(&a.get_y_fp16(),&b.get_y_fp16())
        }
    }

    pub fn active_objects_indices_sorted_pri_y(&self) -> impl Iterator<Item = usize> {
        let t_indices:Vec<usize> = (0..self.objects.len())
            .filter(|b| self.object(&type_object_from_u8(*b as u8)).is_active())
            .sorted_unstable_by(|a,b| self.compare_pri_y(&self.object(&type_object_from_u8(*a as u8)),&self.object(&type_object_from_u8(*b as u8))))
            .collect_vec();
        t_indices.into_iter()
    }
    
    pub fn mut_active_objects(&mut self) -> impl Iterator<Item = (usize,&mut Sprite)> {
        (0..self.objects.len()).zip(self.objects.iter_mut()).filter(|(_,b)| b.active)
    }

    pub fn picture(&self) -> &[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE] {
        &self.picture_buffer
    }
    
    pub fn mut_picture(&mut self) -> &mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE] {
        &mut self.picture_buffer
    }
    
    pub fn priority(&self) -> &[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE] {
        &self.priority_buffer
    }

    pub fn back_buffer(&self) -> &[u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE] {
        &self.back_buffer
    }
    
    pub fn mut_back_buffer(&mut self) -> &mut [u8;SCREEN_WIDTH_USIZE*SCREEN_HEIGHT_USIZE] {
        &mut self.back_buffer
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

    pub fn clear_key(&mut self,key:&TypeController) {
        if let Some(controller) = self.controllers.get(&key.get_value()) {
            let mut new_keys = [AgiKeyCodes::Enter;256];
            let mut new_cnt:usize=0;
            for a in 0..self.key_len {
                if self.key_buffer[a]!=*controller {
                    new_keys[new_cnt]=self.key_buffer[a];
                    new_cnt+=1;
                }
            }
            self.key_buffer=new_keys;
            self.key_len=new_cnt;
        }

    }

    pub fn key_pressed(&mut self,code:&AgiKeyCodes) {
        if self.key_len<256 {
            self.key_buffer[self.key_len]=*code;
            self.key_len+=1;
        }
    }

    pub fn is_key_pressed(&self,code:&AgiKeyCodes) -> bool {
        for a in 0..self.key_len {
            if self.key_buffer[a]==*code {
                return true;
            }
        }
        false
    }

    pub fn is_controller_pressed(&self,key:&TypeController) -> bool {
        if let Some(controller) = self.controllers.get(&key.get_value()) {
            for a in 0..self.key_len {
                if self.key_buffer[a]==*controller {
                    return true;
                }
            }
        }
        false
    }

    pub fn set_controller(&mut self,c:&TypeController,keycode:&AgiKeyCodes) {
        self.controllers.insert(c.get_value(), *keycode);
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
        .finish()
    }
}

#[derive(FromToRepr,Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum AgiKeyCodes {
    Left = 0x4B00,
    Right = 0x4D00,
    Up = 0x4800,
    Down = 0x5000,
    Escape = 0x001B,
    Space = 0x0020,
    Enter = 0x000D,
    Tab = 0x0009,
    Backspace = 0x0008,
    A  = 0x0061,
    B  = 0x0062,
    C  = 0x0063,
    D  = 0x0064,
    E  = 0x0065,
    F  = 0x0066,
    G  = 0x0067,
    H  = 0x0068,
    I  = 0x0069,
    J  = 0x006A,
    K  = 0x006B,
    L  = 0x006C,
    M  = 0x006D,
    N  = 0x006E,
    O  = 0x006F,
    P  = 0x0070,
    Q  = 0x0071,
    R  = 0x0072,
    S  = 0x0073,
    T  = 0x0074,
    U  = 0x0075,
    V  = 0x0076,
    W  = 0x0077,
    X  = 0x0078,
    Y  = 0x0079,
    Z  = 0x007A,
    _0 = 0x0030,
    _1 = 0x0031,
    _2 = 0x0032,
    _3 = 0x0033,
    _4 = 0x0034,
    _5 = 0x0035,
    _6 = 0x0036,
    _7 = 0x0037,
    _8 = 0x0038,
    _9 = 0x0039,
    F1 = 0x3B00,
    F2 = 0x3C00,
    F3 = 0x3D00,
    F4 = 0x3E00,
    F5 = 0x3F00,
    F6 = 0x4000,
    F7 = 0x4100,
    F8 = 0x4200,
    F9 = 0x4300,
    F10= 0x4400,
}

impl AgiKeyCodes {
    pub fn is_ascii(&self) -> bool {
        u16::from(*self) < 256
    }

    pub fn get_ascii(&self) -> u8 {
        (u16::from(*self)&0xFF) as u8
    }
}

pub struct Interpretter {
    pub resources:GameResources,
    pub state:LogicState,
    pub stack:Vec<LogicExecutionPosition>,
    pub keys:Vec<AgiKeyCodes>,
    pub breakpoints:HashMap<LogicExecutionPosition,bool>,
    pub command_input_string:String,
    pub started:u64     // 1/20 ticks since started, so seconds is divdide this by 20
}

impl Interpretter {
    pub fn new(base_path:&'static str,version:&str) -> Result<Interpretter,String> {
        let resources = GameResources::new(base_path,version)?;
        let mut i = Interpretter {
            resources,
            state: LogicState::new(),
            stack: Vec::new(),
            keys: Vec::new(),
            breakpoints: HashMap::new(),
            command_input_string: String::new(),
            started:0,
        };
        i.state.set_var(&VAR_TIME_DELAY,2);
        i.state.initialise_rooms(&i.resources.objects.objects);
        Interpretter::new_room(&i.resources,&mut i.state,0);

        Ok(i)
    }

    pub fn is_paused(&self) -> bool {
        !self.stack.is_empty() && !self.stack[self.stack.len()-1].is_input_request()
    }

    pub fn do_call(breakpoints:&mut HashMap<LogicExecutionPosition,bool>,resources:&GameResources,stack:&mut Vec<LogicExecutionPosition>,state:&mut LogicState, logics:&HashMap<usize,LogicResource>,resume:bool,single_step:bool) {
        let mut resume = resume;
        while !stack.is_empty() {
            let stack_pos = stack.len()-1;
            let entry = stack[stack_pos];
            let logic_sequence = logics[&entry.get_logic()].get_logic_sequence();
            let actions = logic_sequence.get_operations();
            let mut exec = entry;
            loop {
                if !resume {
                    if breakpoints.contains_key(&exec) && !single_step {
                        if breakpoints[&exec] {
                            breakpoints.remove(&exec);
                        }
                        stack[stack_pos]=exec;
                        return;
                    }
                }
                resume=false;
                match Interpretter::interpret_instructions(resources,state,&exec,actions,logic_sequence) {
                    Some(newpc) => {
                        if newpc.is_input_request() {
                            stack[stack_pos]=newpc;
                            return;
                        } else if newpc.is_call(entry.get_logic()) {
                            stack[stack_pos]=exec.next();
                            stack.push(newpc);
                            if single_step {
                                return;
                            }
                            break;
                        } else {
                            exec = newpc;
                            if single_step {
                                stack[stack_pos]=exec;
                                return;
                            }
                        }
                    },
                    None => {
                        stack.pop();
                        if single_step {
                            return;
                        }
                        if state.get_new_room()!=0 {
                            stack.clear();  // new_room short circuits the interpretter cycle
                        }
                        break;
                    },
                }
            }
        }

    }

    pub fn call(breakpoints:&mut HashMap<LogicExecutionPosition,bool>,resources:&GameResources,stack:&mut Vec<LogicExecutionPosition>,state:&mut LogicState,logic_file:usize, logics:&HashMap<usize,LogicResource>,resume:bool,single_step:bool) {
        if stack.is_empty() {
            stack.push(LogicExecutionPosition::new(logic_file,0));
        }
        Interpretter::do_call(breakpoints, resources, stack, state,logics,resume,single_step);
    }

    pub fn key_code_pressed(&mut self,key_code:AgiKeyCodes) {
        self.keys.push(key_code);
    }
    
    pub fn clear_keys(&mut self) {
        self.keys.clear();
    }

    pub fn run(&mut self,resume:bool,single_step:bool) {

        let mut resuming = !self.stack.is_empty();
        let mutable_state = &mut self.state;
        let mutable_stack = &mut self.stack;

        // delay (increment time by delay for now, in future, we should actually delay!)
        self.started+=(mutable_state.get_var(&VAR_TIME_DELAY)+1) as u64;

        if !resuming {
            mutable_state.set_flag(&FLAG_COMMAND_ENTERED, false);
            mutable_state.set_flag(&FLAG_SAID_ACCEPTED_INPUT, false);
        }

        if mutable_state.is_input_enabled() {
            let (done,new_string) = command_input(mutable_state, self.command_input_string.clone(),20,&String::from(">"),&self.resources,0,22,15,0,false);    // not sure if attributes are affected for this
            self.command_input_string = new_string;
            if done && self.command_input_string.len()>0 {
                // parse and clear input string
                parse_input_string(mutable_state, self.command_input_string.clone(), &self.resources);
                self.command_input_string.clear();
            }
        }
        
        // poll keyb/joystick
        mutable_state.clear_keys();
        for k in &self.keys {
            if k.is_ascii() {
                mutable_state.set_var(&VAR_CURRENT_KEY,k.get_ascii());
            }
            mutable_state.key_pressed(k);
        }

        if !resuming {
            // if program.control (EGO dir = var(6))
            // if player.control (var(6) = EGO dir)
            if mutable_state.is_ego_player_controlled() {

                // emulate walking behaviour
                let mut d = mutable_state.get_var(&VAR_EGO_MOTION_DIR);
                for k in &self.keys {
                    d = match k {
                        AgiKeyCodes::Left => if d==7 { 0 } else { 7 },
                        AgiKeyCodes::Right => if d==3 { 0 } else { 3 },
                        AgiKeyCodes::Up => if d==1 { 0 } else { 1 },
                        AgiKeyCodes::Down => if d==5 { 0 } else { 5 },
                        _ => d,
                    }
                }

                mutable_state.set_var(&VAR_EGO_MOTION_DIR, d);
            } else {
                let d = mutable_state.get_var(&VAR_EGO_MOTION_DIR);
                mutable_state.mut_object(&OBJECT_EGO).set_direction(d);
            }
            // For all objects wich animate.obj,start_update and draw
            //  recaclc dir of movement
            update_sprites(&self.resources,mutable_state);
            update_anims(&self.resources,mutable_state);

            // If score has changed(var(3)) or sound has turned off/on (flag(9)), update status line
            //show VAR_CURRENT_SCORE out of VAR_MAXIMUM_SCORE .... SOUND ON/OFF

            mutable_state.set_var(&VAR_FREE_PAGES,255);
            let mut since_started = self.started/20;
            let days = since_started/(60*60*24);
            since_started%=24*60*60;
            let hours = since_started/(60*60);
            since_started%=60*60;
            let minutes = since_started/(60);
            since_started%=60;
            let seconds = since_started;

            mutable_state.set_var(&VAR_DAYS,days as u8);
            mutable_state.set_var(&VAR_HOURS,hours as u8);
            mutable_state.set_var(&VAR_MINUTES,minutes as u8);
            mutable_state.set_var(&VAR_SECONDS,seconds as u8);
        }
        
        loop {

            if !resuming {
                // Execute Logic 0
                mutable_state.reset_new_room();
            }
            
            Interpretter::call(&mut self.breakpoints,&self.resources,mutable_stack,mutable_state, 0, &self.resources.logic,resume,single_step);
            if !mutable_stack.is_empty() {
                break;
            } else {
                resuming=false;
            }

            // dir of EGO <- var(6)
            if mutable_state.is_ego_player_controlled() {
                let d = mutable_state.get_var(&VAR_EGO_MOTION_DIR);
                mutable_state.mut_object(&OBJECT_EGO).set_direction(d);
            } else {
                let d = mutable_state.object(&OBJECT_EGO).get_direction();
                mutable_state.set_var(&VAR_EGO_MOTION_DIR, d);
            }
            mutable_state.set_var(&VAR_OBJ_EDGE, 0);
            mutable_state.set_var(&VAR_OBJ_TOUCHED_BORDER, 0);
            mutable_state.set_flag(&FLAG_ROOM_FIRST_TIME, false);
            mutable_state.set_flag(&FLAG_RESTART_GAME, false);
            mutable_state.set_flag(&FLAG_RESTORE_GAME, false);
            // update all controlled objects on screen
            // if new room issued, rerun logic
            if mutable_state.get_new_room()!=0 {
                Interpretter::new_room(&self.resources,mutable_state,mutable_state.get_new_room());
            } else {
                break;
            }
        }

        render_sprites(&self.resources,mutable_state,false);

        mutable_state.render_final_buffer();
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
            ConditionOperation::Posn((obj,num1,num2,num3,num4)) => is_left_edge_in_box(resources,state,obj,num1,num2,num3,num4),
            ConditionOperation::Controller((key,)) => { let pressed=state.is_controller_pressed(key); state.clear_key(key); pressed },
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
            ConditionOperation::CenterPosn((obj,num1,num2,num3,num4)) => is_center_edge_in_box(resources, state, obj, num1, num2, num3, num4),
            ConditionOperation::RightPosn((obj,num1,num2,num3,num4)) => is_right_edge_in_box(resources,state,obj,num1,num2,num3,num4),
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
            obj.set_cycling(false);
            //obj.set_priority_auto();    // ??
            //obj.reset();//  (may not be needed)
            obj.set_step_size(1);
            obj.set_step_time(1);
            obj.set_cycle_time(1);

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
        let v = usize::from(state.object(&OBJECT_EGO).get_view());
        if resources.views.contains_key(&v) {
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
        }

        state.set_var(&VAR_EGO_EDGE,0);
        state.set_flag(&FLAG_COMMAND_ENTERED,false);
        state.set_flag(&FLAG_ROOM_FIRST_TIME,true);
        // score<- var 3
    }

    fn append_expansion_to_message(state:&LogicState,resources:&GameResources,file:usize,new_string:&mut String,num:u8,n_kind:u8) {
        new_string.push_str(match n_kind {
            b'v' => state.get_var(&TypeVar::from(num)).to_string(),
            b'm' => Interpretter::decode_message_from_resource(state, resources, file, &TypeMessage::from(num)),
            b'o' => resources.objects.objects[num as usize].name.clone(),
            b'w' => state.get_parsed_word_num(num),
            b's' => state.get_string(&TypeString::from(num)).clone(),
            b'g' => Interpretter::decode_message_from_resource(state, resources, 0, &TypeMessage::from(num)),
            _ => todo!(),
        }.as_str());
    }

    fn decode_message_from_resource(state:&LogicState,resources:&GameResources,file:usize,message:&TypeMessage) -> String {
        let mut new_string=String::from("");
        let mut c_state = 0;
        let mut n_kind = b' ';
        let mut num = 0;
        let m = &resources.logic[&file].get_logic_messages().strings[state.get_message(message) as usize];
        let b = m.bytes();
        for c in b {
            match c_state {
                0 => if c == b'%' { c_state=1; } else { new_string.push(c as char); },
                1 => match c {
                    b'v' | b'm' | b'o' | b'w' | b's' | b'g'  => { n_kind=c; num=0; c_state=2; },
                    _ => todo!(),
                },
                2 => if c>=b'0' && c<=b'9' { num*=10; num+=c-b'0'; } else 
                {
                    Self::append_expansion_to_message(state, resources, file, &mut new_string,num,n_kind);
                    new_string.push(c as char); c_state=0;
                }
                _ => todo!(),
            }
        }
        if c_state == 2 {
            // Deal with the case where the number is at the end of the string
            Self::append_expansion_to_message(state, resources, file, &mut new_string,num,n_kind);
        }
        new_string
    }

    fn handle_window_request(resources:&GameResources,state:&mut LogicState,pc:&LogicExecutionPosition,m:String,x:u8,y:u8,w:u8) -> Option<LogicExecutionPosition> {
        if state.displayed != m {
            state.displayed = m.clone();
            Self::close_windows(resources, state);
        } 
        if !Self::is_window_open(state) {
            Self::display_window(resources, state, m.as_str(),x,y,w);
            return Some(pc.user_input());
        } else {
            // todo check dialog flag timer thing, for now, any key to exit
            if !state.get_flag(&FLAG_LEAVE_WINDOW_OPEN) {
                let key_pressed = state.is_key_pressed(&AgiKeyCodes::Enter) || state.is_key_pressed(&AgiKeyCodes::Escape);
                state.clear_keys();
                if key_pressed {
                    Self::close_windows(resources, state);
                } else {
                    return Some(pc.user_input());
                }
            } else {
                println!("Leave Window Open @{} v21: {}",pc,state.get_var(&VAR_MESSAGE_WINDOW_TIMER));
            }
        }
        Some(pc.next())
    }

    fn interpret_instruction(resources:&GameResources,state:&mut LogicState,pc:&LogicExecutionPosition,action:&ActionOperation,logic_sequence:&LogicSequence) -> Option<LogicExecutionPosition> {

        match action {
            // Not complete
            ActionOperation::Sound((_num,flag)) => /* TODO RAGI  - for now, just pretend sound finished*/ {/*println!("TODO : Sound@{}",pc); */state.set_flag(flag,true);},
            ActionOperation::StopSound(()) => /* TODO RAGI - for now, since we complete sounds straight away, does nothing */ {/*println!("TODO : StopSound@{}",pc);*/},
            ActionOperation::SetGameID((m,)) => /* TODO RAGI - if needed */{let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); println!("TODO : SetGameID@{} {:?}",pc,m);},
            ActionOperation::ConfigureScreen((a,b,c)) => /* TODO RAGI */ { println!("TODO : ConfigureScreen@{} {:?},{:?},{:?}",pc,a,b,c);},
            ActionOperation::SetMenu((m,)) => /* TODO RAGI */ { let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); println!("TODO : SetMenu@{} {}",pc,m); },
            ActionOperation::SetMenuMember((m,c)) => /* TODO RAGI */{ let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); println!("TODO : SetMenuMember@{} {} {}",pc,m,state.get_controller(c)); },
            ActionOperation::SubmitMenu(()) => /* TODO RAGI */ { println!("TODO : SubmitMenu@{}",pc)},
            ActionOperation::TraceInfo((num1,num2,num3)) => /* TODO RAGI */ { println!("TODO : TraceInfo@{} {} {} {}",pc,state.get_num(num1),state.get_num(num2),state.get_num(num3)); }
            ActionOperation::DisableMember((c,)) => /* TODO RAGI */ println!("TODO : DisableMember@{} {}",pc, state.get_controller(c)),
            ActionOperation::CancelLine(()) => /* TODO RAGI */ println!("TODO : CancelLine@{}",pc),
            ActionOperation::ForceUpdate((o,)) => /* TODO RAGI */ println!("TODO : ForceUpdate@{} {:?}",pc,o),
            ActionOperation::ShakeScreen((num,)) => /* TODO RAGI */ println!("TODO : ShakeScreen@{} {:?}",pc,num),
            ActionOperation::PrintAtV0((m,x,y,)) => /* TODO RAGI */ { let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); println!("TODO : PrintAtV0@{} {} {},{}",pc,m,state.get_num(x),state.get_num(y)); },
            ActionOperation::Block((a,b,c,d)) => /* TODO RAGI */ { println!("TODO : Block@{} {},{},{},{}",pc,state.get_num(a),state.get_num(b),state.get_num(c),state.get_num(d)); },
            ActionOperation::Unblock(()) => /* TODO RAGI */ println!("TODO : Unblock@{}",pc),
            ActionOperation::OpenDialog(()) => /* TODO RAGI */ println!("TODO : OpenDialog@{}",pc),
            ActionOperation::CloseDialog(()) => /* TODO RAGI */ println!("TODO : CloseDialog@{}",pc),
            ActionOperation::SetPriBase((num,)) => /* TODO RAGI */ println!("TODO : SetPriBase@{} {}",pc,state.get_num(num)),
            ActionOperation::HoldKey(()) => /* TODO RAGI */ println!("TODO : HoldKey@{}",pc),
            ActionOperation::ReleaseKey(()) => /* TODO RAGI */ println!("TODO : ReleaseKey@{}",pc),
            ActionOperation::PushScript(()) => /* TODO RAGI */ println!("TODO : PushScript@{}",pc),
            ActionOperation::PopScript(()) => /* TODO RAGI */ println!("TODO : PopScript@{}",pc),


            

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
                    new_pc = pc.jump(logic_sequence,goto_if_false);
                } else {
                    new_pc = pc.next();
                }
                if need_tick {
                    return Some(new_pc.user_input());
                } else {
                    return Some(new_pc);
                }
            },
            ActionOperation::Goto((goto,)) => {
                return Some(pc.jump(logic_sequence, goto))
            },
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
            ActionOperation::SetView((obj,num)) => {let n=state.get_num(num); state.mut_object(obj).set_view(n,resources); },
            ActionOperation::SetViewV((obj,var)) => {let n=state.get_var(var); state.mut_object(obj).set_view(n,resources); },
            ActionOperation::ObserveObjs((obj,)) => state.mut_object(obj).set_observing(true),
            ActionOperation::LIndirectN((var,num)) => {let v = &TypeVar::from(state.get_var(var)); state.set_var(v,state.get_num(num)); },
            ActionOperation::Increment((var,)) => state.set_var(var,state.get_var(var).saturating_add(1)),
            ActionOperation::Decrement((var,)) => state.set_var(var,state.get_var(var).saturating_sub(1)),
            ActionOperation::GetPosn((obj,var1,var2)) => { state.set_var(var1, state.object(obj).get_x()); state.set_var(var2, state.object(obj).get_y()); },
            ActionOperation::StopCycling((obj,)) => state.mut_object(obj).set_cycling(false),
            ActionOperation::PreventInput(()) => state.set_input(false),
            ActionOperation::SetHorizon((num,)) => state.set_horizon(state.get_num(num)),
            ActionOperation::Reposition((obj,var1,var2)) => {let dx=state.get_var(var1); let dy=state.get_var(var2); state.mut_object(obj).adjust_x_via_delta(dx); state.mut_object(obj).adjust_y_via_delta(dy); shuffle(state,resources,obj); },
            ActionOperation::SetPriority((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_priority(n); },
            ActionOperation::SetLoop((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_loop(n); },
            ActionOperation::SetCel((obj,num)) => { let n=state.get_num(num); state.mut_object(obj).set_cel(n); },
            ActionOperation::DrawPic((var,)) => { let n = state.get_var(var); resources.pictures[&usize::from(n)].render_to(&mut state.picture_buffer,&mut state.priority_buffer).unwrap(); },
            ActionOperation::ShowPic(()) => {
                let dpic = double_pic_width(state.picture());
                for y in 0usize..PIC_HEIGHT_USIZE {
                    for x in 0usize..PIC_WIDTH_USIZE*2 {
                        state.back_buffer[x+y*SCREEN_WIDTH_USIZE] = dpic[x+y*SCREEN_WIDTH_USIZE];
                    }
                }
                // Clear textbuffer on showpic
                let start=0;
                let end = PIC_HEIGHT_USIZE;
                let col = 255;
                for y in start..=end {
                    for x in 0usize..SCREEN_WIDTH_USIZE {
                        state.text_buffer[x+y*SCREEN_WIDTH_USIZE] = col;
                    }
                }
                state.set_flag(&FLAG_LEAVE_WINDOW_OPEN, false); // Aparantly original interpretter did this
            },
            ActionOperation::ClearLines((num1,num2,num3)) => {
                let start=usize::from(state.get_num(num1) * 8);
                let end = usize::from(state.get_num(num2) * 8)+7;
                let input_col = state.get_num(num3);
                let col;
                if state.text_mode {
                    col=0;
                } else {
                    col = if input_col==0 { 0 } else { 15 };
                }
                for y in start..=end {
                    for x in 0usize..SCREEN_WIDTH_USIZE {
                        state.text_buffer[x+y*SCREEN_WIDTH_USIZE] = col;
                    }
                }
            },
            ActionOperation::SetString((s,m)) => { let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); state.set_string(s,m.as_str()); },
            ActionOperation::Draw((obj,)) => if !state.object(obj).get_visible() { shuffle(state,resources,obj); state.mut_object(obj).set_visible(true); },
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
            ActionOperation::Display((num1,num2,m)) => { let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); let x=state.get_num(num2); let y=state.get_num(num1); Self::display_text(resources,state,x,y,&m,state.get_ink(),state.get_paper()); },
            ActionOperation::DisplayV((var1,var2,var3)) => { let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, &TypeMessage::from(state.get_var(var3))); let x=state.get_var(var2); let y=state.get_var(var1); Self::display_text(resources,state,x,y,&m,state.get_ink(),state.get_paper()); },
            ActionOperation::ReverseLoop((obj,flag)) => { state.set_flag(flag, false); state.mut_object(obj).set_one_shot_reverse(flag); },
            ActionOperation::Random((num1,num2,var)) => { let r = state.get_random(num1,num2); state.set_var(var,r); },
            ActionOperation::Set((flag,)) => state.set_flag(flag, true),
            ActionOperation::SetV((var,)) => { let flag=&TypeFlag::from(state.get_var(var)); state.set_flag(flag, true); },
            ActionOperation::TextScreen(()) => state.set_text_mode(true),
            ActionOperation::GetString((s,m,num1,num2,num3)) => {
                // This actually halts interpretter until the input string is entered
                let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); 
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
                let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); 
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
            ActionOperation::SetCursorChar((m,)) => { let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); state.set_prompt(&m); },
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
            ActionOperation::ObjectOnLand((obj,)) => state.mut_object(obj).set_restrict_to_land(),
            ActionOperation::Wander((obj,)) => {
                let dist=state.rng.gen_range(6u8..=50u8);
                state.mut_object(obj).set_wander(dist);
                if *obj==OBJECT_EGO {
                    state.set_program_control()
                }
            }
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
            // TODO investigate, Position and RepositionTo act the same, should they (technically reposition clears old object first, but sprites in ragi don't work that way)
            ActionOperation::Position((obj,num1,num2)) => { let x=state.get_num(num1); let y=state.get_num(num2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y); shuffle(state,resources,obj); },
            ActionOperation::RepositionTo((obj,num1,num2)) => { let x=state.get_num(num1); let y=state.get_num(num2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y);  shuffle(state,resources,obj); },
            ActionOperation::RepositionToV((obj,var1,var2)) => {let x=state.get_var(var1); let y=state.get_var(var2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y);  shuffle(state,resources,obj); },
            ActionOperation::PositionV((obj,var1,var2)) => { let x=state.get_var(var1); let y=state.get_var(var2); state.mut_object(obj).set_x(x); state.mut_object(obj).set_y(y);  shuffle(state,resources,obj); },
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
                let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m);
                return Interpretter::handle_window_request(resources, state, pc, m, 255, 255, 255);
            },
            ActionOperation::PrintV((var,)) => { 
                let m=&TypeMessage::from(state.get_var(var));
                let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m);
                return Interpretter::handle_window_request(resources, state, pc, m, 255, 255, 255);
            },
            ActionOperation::PrintAtV1((m,y,x,w)) => { 
                let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); 
                let x = state.get_num(x);
                let y = state.get_num(y);
                let w = state.get_num(w);
                return Interpretter::handle_window_request(resources,state,pc,m,x,y,w);
            },
            ActionOperation::PrintAtVV1((var,y,x,w)) => { 
                let m=&TypeMessage::from(state.get_var(var));
                let m = Interpretter::decode_message_from_resource(state, resources, pc.logic_file, m); 
                let x = state.get_num(x);
                let y = state.get_num(y);
                let w = state.get_num(w);
                return Interpretter::handle_window_request(resources,state,pc,m,x,y,w);
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
                    Self::display_window(resources, state, m.as_str(),255,255,255);
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
            },
            ActionOperation::GetPriority((obj,var)) => state.set_var(var,state.object(obj).get_priority()),
            ActionOperation::LIndirectV((var1,var2)) => {let v = &TypeVar::from(state.get_var(var1)); state.set_var(v,state.get_var(var2)); },
            ActionOperation::ReleaseLoop((obj,)) => state.mut_object(obj).set_fixed_loop(false),
            ActionOperation::UnanimateAll(()) => state.unanimate_all(),
            ActionOperation::GetRoomV((item,var)) => { let item = &TypeItem::from(state.get_var(item));let loc = state.get_item_room(item); state.set_var(var,loc); }
            ActionOperation::GetDir((obj,var)) => { let dir = state.object(obj).get_direction(); state.set_var(var,dir); },
            ActionOperation::SetLoopV((obj,var)) => { let n=state.get_var(var); state.mut_object(obj).set_loop(n); },
            ActionOperation::AddToPicV((var1,var2,var3,var4,var5,var6,var7)) => /* TODO RAGI */ {
                let view=state.get_var(var1);
                let cloop=state.get_var(var2);
                let cel=state.get_var(var3);
                let x=state.get_var(var4) as usize;
                let y=state.get_var(var5) as usize;
                let rpri=state.get_var(var6);
                let margin=state.get_var(var7);
                render_view_to_pic(resources, state, view, cloop, cel, x, y, rpri, margin);
            },
            ActionOperation::RIndirect((var1,var2)) => {let v = &TypeVar::from(state.get_var(var2)); state.set_var(var1,state.get_var(v)); },
            ActionOperation::NormalCycle((obj,)) => state.mut_object(obj).set_normal_cycle(),
            ActionOperation::ReverseCycle((obj,)) => state.mut_object(obj).set_reverse_cycle(),
            ActionOperation::SetDir((obj,var)) => { let dir = state.get_var(var); state.mut_object(obj).set_direction(dir); },
            ActionOperation::SetKey((a,b,c)) => 
            {
                let code:u16 = b.get_value().into();
                let code=code<<8;
                let code = code | (a.get_value() as u16);
                if let Ok(keycode) = AgiKeyCodes::try_from(code) {
                    state.set_controller(c,&keycode);
                } else {
                    // Find appropriate AGIKey for 
                    println!("Unhandled KeyCode : SetKey@{} {:?},{:?},{:?}",pc,a,b,c);
                }
            },
            ActionOperation::Pause(()) => return Interpretter::handle_window_request(resources, state, pc,String::from("      Game paused.\nPress Enter to continue."), 255, 255, 255),

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
        let mut y=y*8;
        for l in s.as_bytes() {
            if *l == b'\n' {
                y+=8;
                x=0;
            } else {
                Self::render_glyph(resources, state, x, y, *l,ink,paper);
                x+=8;
            }
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

    pub fn display_window(resources:&GameResources,state:&mut LogicState, message:&str,x:u8,y:u8,w:u8) {

        // compute window size
        let mut max_width=0;
        let mut width=0u16;
        let mut word_len=0;
        let mut height=1u8;
        let mut iter = 0;
        let bytes = message.as_bytes();
        let mut splits=[999usize;40];
        let n = if x == 255 { 30 } else { 38-x };
        let w = if w==255 || w==0 { n as u16 } else { w as u16};
        while iter < bytes.len() {
            let c=bytes[iter];
            if width>=w {
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

        let y0 = if x == 255 { 10 - height/2 } else { y };
        let x0 = if x == 255 { 20 - max_width/2 } else { x as u16 };
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

    pub fn interpret_instructions(resources:&GameResources,state:&mut LogicState,pc:&LogicExecutionPosition,actions:&[LogicOperation],logic_sequence:&LogicSequence) -> Option<LogicExecutionPosition> {
        Interpretter::interpret_instruction(resources, state, pc, &actions[pc.program_counter].action,logic_sequence)
    }

    pub fn set_breakpoint(&mut self,file:usize,pc:usize,temporary:bool) {
        self.breakpoints.insert(LogicExecutionPosition::new(file,pc), temporary);
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
        let c = state.key_buffer[a].get_ascii();
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
    Interpretter::display_text(resources, state, x, y, &to_show,ink,paper);
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
                    let distance = state.rng.gen_range(6u8..=50u8);
                    state.mut_object(obj_num).set_direction(direction);
                    state.mut_object(obj_num).wander_distance=FP16::from_num(distance);
                } else {
                    let s = state.object(obj_num).get_step_size();
                    let t = state.object(obj_num).wander_distance.saturating_sub(s);
                    state.mut_object(obj_num).wander_distance=t;
                    if (state.object(obj_num)).wander_distance==FP16::from_num(0) {
                        state.mut_object(obj_num).set_direction(0);
                    }
                }
            },
            SpriteMotion::MoveObj => {
                let x=FP32::from(state.object(obj_num).get_x_fp16());
                let y=FP32::from(state.object(obj_num).get_y_fp16());
                let ex=FP32::from(state.object(obj_num).get_end_x());
                let ey=FP32::from(state.object(obj_num).get_end_y());
                let dx=ex.int()-x.int();
                let dy=ey.int()-y.int();
                let s = FP32::from(state.object(obj_num).get_step_size());
                let direction = if dx.abs() <= s && dy.abs() <= s {
                    0
                } else {
                    let sx = dx.signum();
                    let sy = dy.signum();
                    get_direction_from_delta(sx.to_num(), sy.to_num())
                };
                state.mut_object(obj_num).set_direction(direction);
                if direction==0 || !state.object(obj_num).has_moved() {
                    let mflag = state.object(obj_num).move_flag;
                    state.set_flag(&mflag, true);
                    state.mut_object(obj_num).clear_move();
                    state.mut_object(obj_num).restore_step_size();
                    if obj_num.get_value()==OBJECT_EGO.get_value() {
                        state.set_player_control();
                    }
                }
            },
            SpriteMotion::FollowEgo => {
                let x=FP32::from(state.object(obj_num).get_x_fp16());
                let y=FP32::from(state.object(obj_num).get_y_fp16());
                let ex=FP32::from(state.object(&OBJECT_EGO).get_x_fp16());
                let ey=FP32::from(state.object(&OBJECT_EGO).get_y_fp16());
                let dx=ex.int()-x.int();
                let dy=ey.int()-y.int();
                let s = FP32::from(state.object(obj_num).get_step_size());
                let direction = if dx.abs() <= s || dy.abs() <= s {
                    0
                } else {
                    let sx = dx.signum();
                    let sy = dy.signum();
                    get_direction_from_delta(sx.to_num(), sy.to_num())
                };
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
        state.set_var(&VAR_OBJ_TOUCHED_BORDER,obj_num.get_value());
    }
}

pub fn pri_slice_for_baseline(state:&LogicState,x:usize,y:usize,w:usize) -> &[u8] {

    let s = x+y*PIC_WIDTH_USIZE;
    let e = s + w;
    &state.priority_buffer[s..e]
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
    let slice = pri_slice_for_baseline(state, tx, ty, w);
    for pri in slice {
        let pri=*pri;
        water&=pri==3;
        if pri == 3 && obj.is_restricted_to_land() {
            blocked=true;
        }
        if pri != 3 && obj.is_restricted_to_water() {
            blocked=true;
        }
        if obj.priority != 15 && pri == 0 {
            blocked=true;
        }
        if obj.priority != 15 && pri == 1 {
            if obj.is_restricted_by_blocks() {
                blocked=true;
            }
        }
        if pri == 2 {
            signal=true;
        }
    }

    if blocked {
        return (false,nx,ny,water,signal);
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

    for num in state.active_objects_indices_sorted_pri_y() {
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

    shuffle_with_vars(state,resources);

    if rpri == 0 {
        println!("render_view_to_pic - need to fetch priority for 0");
    }

    for yy in 0..h {
        for xx in 0..w {
            let col = if mirror {d[(w-xx-1)+yy*w]} else {d[xx+yy*w] };
            let sy = yy+y;
            if col != t && h<=sy {
                let sx = xx+x;
                let sy = sy-h;
                let picture_coord = sx+sy*PIC_WIDTH_USIZE;
                let coord = sx*2+sy*SCREEN_WIDTH_USIZE;
                let pri = fetch_priority_for_pixel_rendering(state,sx,sy);
                if pri <= rpri {
                    state.mut_picture()[picture_coord]=col; // render to picture buffer and back_buffer (in case show.pic has not yet occured)
                    state.mut_back_buffer()[coord]=col;
                    state.mut_back_buffer()[coord+1]=col;
                }
            }
        }
    }
    // render control value
    if margin<4 {
        println!("TODO : RENDER BOX, NOT just baseline for MARGIN");
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

fn shuffle_with_vars(state:&mut LogicState,resources:&GameResources) {
    println!("TO IMPLEMENT - SHUFFLE");
}

fn shuffle(state:&mut LogicState,resources:&GameResources,obj:&TypeObject) {
    shuffle_with_vars(state, resources);    // TODO expand object into needed vars
}