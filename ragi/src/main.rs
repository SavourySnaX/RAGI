
use helpers::conv_rgba;
use logic::{LogicResource, LogicSequence, LogicState, LogicExecutionPosition, TypeFlag, GameResources, TypeVar, render_sprites, update_sprites};


use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::rect::Rect;

use std::collections::HashMap;

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
        interpretter.clear_keys();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(code), ..} => {
                    interpretter.key_code_pressed(code);
                }
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
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

struct Interpretter {
    resources:GameResources,
    state:LogicState,
    stack:Vec<LogicExecutionPosition>,
}

impl Interpretter {
    pub fn new(base_path:&'static str) -> Result<Interpretter,String> {
        let resources = GameResources::new(base_path)?;
        return Ok(Interpretter {
            resources,
            state: LogicState::new(),
            stack: Vec::new(),
        });
    }

    pub fn do_call(resources:&GameResources,stack:&mut Vec<LogicExecutionPosition>,state:&mut LogicState, logics:&HashMap<usize,LogicResource>) {

        while !stack.is_empty() {
            let stack_pos = stack.len()-1;
            let entry = stack[stack_pos];
            let logic_sequence = logics[&entry.get_logic()].get_logic_sequence();
            let actions = logic_sequence.get_operations();
            let mut exec = entry;
            loop {
                match logic_sequence.interpret_instructions(resources,state,&exec,&actions) {
                    Some(newpc) => {
                        if newpc.is_input_request() {
                            stack[stack_pos]=newpc;
                            return;
                        } else if newpc.is_call(entry.get_logic()) {
                            stack[stack_pos]=exec.next();
                            stack.push(newpc);
                            break;
                        } else {
                            exec = newpc;
                        }
                    },
                    None => {
                        stack.pop();
                        break;
                    },
                }
            }
        }

    }

    pub fn call(resources:&GameResources,stack:&mut Vec<LogicExecutionPosition>,state:&mut LogicState,logic_file:usize, logics:&HashMap<usize,LogicResource>) {
        if stack.is_empty() {
            stack.push(LogicExecutionPosition::new(logic_file,0));
        }
        Self::do_call(resources, stack, state,logics);
    }

    pub fn key_code_pressed(&mut self,key_code:Keycode) {
        let mutable_state = &mut self.state;

        if (key_code as u32) <256 {
            mutable_state.key_pressed(key_code as u8);
        }
    }
    
    pub fn clear_keys(&mut self) {
        let mutable_state = &mut self.state;

        mutable_state.clear_keys();
    }

    pub fn run(&mut self) {

        let mut resuming = !self.stack.is_empty();
        let mutable_state = &mut self.state;
        let mutable_stack = &mut self.stack;

        if !resuming {
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
        }
        
        loop {

            if !resuming {
                // Execute Logic 0
                mutable_state.reset_new_room();
            }
            
            Self::call(&self.resources,mutable_stack,mutable_state, 0, &self.resources.logic);
            if !mutable_stack.is_empty() {
                return;
            } else {
                resuming=false;
            }

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
