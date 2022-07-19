use dir_resource::ResourceDirectory;
use helpers::{Root, double_width, conv_rgba, dump_png};
use logic::{LogicResource, ActionOperation, LogicOperation, LogicChange, LogicSequence, ConditionOperation, LogicState, LogicExecutionPosition, TypeFlag};
use objects::Objects;
use picture::PictureResource;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use view::ViewResource;
use volume::Volume;
use words::Words;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

fn main() -> Result<(), String> {

    let mut interpretter=Interpretter::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/").unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("rust-sdl2 demo", 640, 400)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let tex_creator = canvas.texture_creator();
    let mut foreground = tex_creator.create_texture(sdl2::pixels::PixelFormatEnum::ARGB8888, sdl2::render::TextureAccess::Streaming, 320, 200).unwrap();


    let (pic,_) = interpretter.pictures.iter().next().unwrap().1.render().unwrap();
    let pic = double_width(&pic);
    let pic = conv_rgba(&pic);

    dump_png("../err.png",320,168, &pic);

    let mut vec:Vec<u8> = vec![0u8;320*200*4];//Vec::new();
    for y in 0usize..168 {
        for x in 0usize..320 {
            for n in 0..4 {
                vec[(x+y*320)*4+n]=pic[(x+y*320)*4+n];
            }
        }
    }

    dump_png("../huh.png",320,200, &vec);

    foreground.update(None, &vec[..], 320*4).unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        interpretter.run();

        canvas.copy(&foreground, None, Rect::new(0,0,320,200)).unwrap();

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

struct Interpretter {
    objects:Objects,
    words:Words,
    views:HashMap<usize,ViewResource>,
    pictures:HashMap<usize,PictureResource>,
    logic:HashMap<usize,LogicResource>,
    state:LogicState,
}

impl Interpretter {
    pub fn new(base_path:&'static str) -> Result<Interpretter,String> {
        let root = Root::new(base_path);
    
        let mut volumes:HashMap<u8,Volume>=HashMap::new();

        let dir = ResourceDirectory::new(root.read_data_or_default("VIEWDIR").into_iter()).unwrap();

        let mut views:HashMap<usize,ViewResource> = HashMap::new();
        views.reserve(256);
        for (index,entry) in dir.into_iter().enumerate() {
            if !entry.empty() {
                if !volumes.contains_key(&entry.volume) {
                    let bytes = root.read_data_or_default(format!("VOL.{}", entry.volume).as_str());
                    volumes.insert(entry.volume, Volume::new(bytes.into_iter())?);
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
                if !volumes.contains_key(&entry.volume) {
                    let bytes = root.read_data_or_default(format!("VOL.{}", entry.volume).as_str());
                    volumes.insert(entry.volume, Volume::new(bytes.into_iter())?);
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
                if !volumes.contains_key(&entry.volume) {
                    let bytes = root.read_data_or_default(format!("VOL.{}", entry.volume).as_str());
                    volumes.insert(entry.volume, Volume::new(bytes.into_iter())?);
                }
                logic.insert(index, LogicResource::new(&volumes[&entry.volume],&entry)?);
            }
        }
        logic.shrink_to_fit();

        return Ok(Interpretter {

            words : Words::new(root.read_data_or_default("WORDS.TOK").into_iter())?,
            objects: Objects::new(&root.read_data_or_default("OBJECT"))?,
            views,
            pictures,
            logic,
            state: LogicState::new(),
        });
    }

    pub fn do_call(state:&mut LogicState, entry:&LogicExecutionPosition, logics:&HashMap<usize,LogicResource>) {
        let logic_sequence = logics[&entry.get_logic()].get_logic_sequence();
        let actions = logic_sequence.get_operations();
        let mut exec = *entry;
        loop {
            match logic_sequence.interpret_instructions(state,&exec,&actions) {
                Some(newpc) => {
                    if newpc.is_call(entry.get_logic()) {
                        Self::do_call(state,&newpc,logics);
                        exec=exec.next();
                    } else {
                        exec = newpc;
                    }
                },
                None => break,
            }
        }
    }

    pub fn call(state:&mut LogicState,logic_file:usize, logics:&HashMap<usize,LogicResource>) {

        let exec = LogicExecutionPosition::new(logic_file,0);
        Self::do_call(state,&exec,logics);
    }

    pub fn run(&mut self) {

        let mutable_state = &mut self.state;

        loop {
            Self::call(mutable_state, 0, &self.logic);
            if !mutable_state.get_flag(&TypeFlag::from(5)) {
                mutable_state.set_flag(&TypeFlag::from(5), false);
                break;
            }
        }
    }
}
