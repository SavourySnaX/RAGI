
use helpers::{conv_rgba, double_pic_width};
use logic::{LogicResource, LogicSequence, LogicState, LogicExecutionPosition, TypeFlag, GameResources, TypeVar, render_sprites, update_sprites};


use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use std::collections::HashMap;
use std::time::Duration;

fn main() -> Result<(), String> {

    let mut interpretter=Interpretter::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/").unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("R.A.G.I", 640, 400)
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
    let mut foreground = tex_creator.create_texture(sdl2::pixels::PixelFormatEnum::ABGR8888, sdl2::render::TextureAccess::Streaming, 320, 200).unwrap();

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

        // Update our texture from our back buffer
        let pic = conv_rgba(interpretter.state.final_buffer());

        let mut vec:Vec<u8> = vec![0u8;320*200*4];//Vec::new();
        for y in 0usize..200 {
            for x in 0usize..320 {
                for n in 0..4 {
                    vec[(x+y*320)*4+n]=pic[(x+y*320)*4+n];
                }
            }
        }

        foreground.update(None, &vec[..], 320*4).unwrap();

        canvas.copy(&foreground, None, Rect::new(0,0,640,400)).unwrap();

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

struct Interpretter {
    resources:GameResources,
    state:LogicState,
}

impl Interpretter {
    pub fn new(base_path:&'static str) -> Result<Interpretter,String> {
        let resources = GameResources::new(base_path)?;
        return Ok(Interpretter {
            resources,
            state: LogicState::new(),
        });
    }

    pub fn do_call(resources:&GameResources,state:&mut LogicState, entry:&LogicExecutionPosition, logics:&HashMap<usize,LogicResource>) {
        let logic_sequence = logics[&entry.get_logic()].get_logic_sequence();
        let actions = logic_sequence.get_operations();
        let mut exec = *entry;
        loop {
            match logic_sequence.interpret_instructions(resources,state,&exec,&actions) {
                Some(newpc) => {
                    if newpc.is_call(entry.get_logic()) {
                        Self::do_call(resources, state,&newpc,logics);
                        exec=exec.next();
                    } else {
                        exec = newpc;
                    }
                },
                None => break,
            }
        }
    }

    pub fn call(resources:&GameResources,state:&mut LogicState,logic_file:usize, logics:&HashMap<usize,LogicResource>) {

        let exec = LogicExecutionPosition::new(logic_file,0);
        Self::do_call(resources, state,&exec,logics);
    }

    pub fn run(&mut self) {

        let mutable_state = &mut self.state;

        // delay
        // clear keybuffer

        mutable_state.set_flag(&TypeFlag::from(2), false);
        mutable_state.set_flag(&TypeFlag::from(4), false);
        // poll keyb/joystick
        // if program.control (EGO dir = var(6))
        // if player.control (var(6) = EGO dir)
        // For all objects wich animate.obj,start_update and draw
        //  recaclc dir of movement
        update_sprites(&self.resources,mutable_state);

        // If score has changed(var(3)) or sound has turned off/on (flag(9)), update status line
        
        loop {
            // Execute Logic 0
            mutable_state.reset_new_room();
            Self::call(&self.resources,mutable_state, 0, &self.resources.logic);
            // dir of EGO <- var(6)
            mutable_state.set_var(&TypeVar::from(5), 0);
            mutable_state.set_var(&TypeVar::from(4), 0);
            mutable_state.set_flag(&TypeFlag::from(5), false);
            mutable_state.set_flag(&TypeFlag::from(6), false);
            mutable_state.set_flag(&TypeFlag::from(12), false);
            // update all controlled objects on screen
            // if new room issued, rerun logic
            if mutable_state.get_new_room()!=0 {
                LogicSequence::new_room(mutable_state,mutable_state.get_new_room());
            } else {
                break;
            }
        }

        render_sprites(&self.resources,mutable_state);

    }
}
