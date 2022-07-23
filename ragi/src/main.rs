
use glow::HasContext;
use helpers::conv_rgba;
use logic::{LogicResource, LogicSequence, LogicState, LogicExecutionPosition, GameResources, render_sprites, update_sprites, VAR_OBJ_TOUCHED_BORDER, VAR_OBJ_EDGE, FLAG_SAID_ACCEPTED_INPUT, FLAG_COMMAND_ENTERED, FLAG_ROOM_FIRST_TIME, FLAG_RESTART_GAME, FLAG_RESTORE_GAME};


use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use imgui::*;
use std::collections::HashMap;
use std::time::Duration;

struct TexturesUi {
    generated_texture: TextureId,
    gl_texture: u32,
}

impl TexturesUi {
    fn new(gl: &glow::Context, textures: &mut Textures<glow::Texture>) -> Self {
        let (generated_texture,gl_texture) = Self::generate(gl, textures);
        Self {
            generated_texture,gl_texture
        }
    }

    fn get_generated_texture(&self) -> TextureId {
        self.generated_texture
    }

    pub fn update(&self,gl:&glow::Context, data:&[u8]) {
        const WIDTH: usize = 320;
        const HEIGHT: usize = 200;
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.gl_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as _, // When generating a texture like this, you're probably working in linear color space
                WIDTH as _,
                HEIGHT as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            )
        }
    }

    /// Generate dummy texture
    fn generate(
        gl: &glow::Context,
        textures: &mut Textures<glow::Texture>,
    ) -> (TextureId,u32) {
        const WIDTH: usize = 320;
        const HEIGHT: usize = 200;

        let mut data = Vec::with_capacity(WIDTH * HEIGHT);
        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                // Insert RGB values
                data.push(i as u8);
                data.push(j as u8);
                data.push((i + j) as u8);
                data.push(255u8);
            }
        }

        let gl_texture = unsafe { gl.create_texture() }.expect("unable to create GL texture");

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(gl_texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as _,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as _, // When generating a texture like this, you're probably working in linear color space
                WIDTH as _,
                HEIGHT as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&data),
            )
        }

        (textures.insert(gl_texture),gl_texture)
    }
}

fn main() -> Result<(), String> {

    let mut interpretter=Interpretter::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/").unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

    let window = video_subsystem.window("R.A.G.I", 640*2, 400*2)
        .position_centered()
        .resizable()
        .opengl()
        .allow_highdpi()
        .build()
        .expect("could not initialize video subsystem");

    let _gl_context = window.gl_create_context().expect("Couldn't create GL context");

    let gl = unsafe {glow::Context::from_loader_function(|s| video_subsystem.gl_get_proc_address(s) as *const _)};

    let mut imgui = Context::create();
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui,&window);

    let mut textures = Textures::<glow::Texture>::default();

    let mut renderer = imgui_glow_renderer::Renderer::initialize(&gl,&mut imgui,&mut textures, true)
        .expect("failed to create renderer");
    let mut event_pump = sdl_context.event_pump()?;

    let textures_ui = TexturesUi::new(&gl,&mut textures);

    'running: loop {
        unsafe {
            gl.clear_color(0.0,0.3,0.3,1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        
        interpretter.clear_keys();
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) { continue; }
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

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        // The rest of the game loop goes here...
        interpretter.run();

        // imgui windows etc
        let pic = conv_rgba(interpretter.state.final_buffer());

        textures_ui.update(&gl,&pic);
        
        let ui = imgui.frame();

        Window::new("MAIN GAME").resizable(false).build(&ui, || {
            Image::new(textures_ui.get_generated_texture(),[640.0,400.0]).build(&ui);
        });

        Window::new("OBJECTS").build(&ui, || {
            for (index,obj) in interpretter.state.active_objects() {
                TreeNode::new(format!("Object {}",index)).flags(if obj.get_visible() {TreeNodeFlags::BULLET} else {TreeNodeFlags::OPEN_ON_ARROW}).build(&ui, || {
                    ui.text_wrapped(format!("{:?}",obj));
                });
            }
        });

        Window::new("LOGIC").build(&ui, || {
            let logic = interpretter.resources.logic.get(&0);
            if !logic.is_none() {
                for s in logic.unwrap().get_disassembly_iterator(&interpretter.resources.words, &interpretter.resources.objects) {
                    ui.text(s);
                }
            }
        });

        imgui_sdl2.prepare_render(&ui,&window);
        let draw_data = ui.render();
        renderer.render(&gl,&textures,draw_data).expect("Renderer failed");

        window.gl_swap_window();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

struct Interpretter {
    pub resources:GameResources,
    pub state:LogicState,
    pub stack:Vec<LogicExecutionPosition>,
    pub keys:Vec<Keycode>,
}

impl Interpretter {
    pub fn new(base_path:&'static str) -> Result<Interpretter,String> {
        let resources = GameResources::new(base_path)?;
        Ok(Interpretter {
            resources,
            state: LogicState::new(),
            stack: Vec::new(),
            keys: Vec::new(),
        })
    }

    pub fn do_call(resources:&GameResources,stack:&mut Vec<LogicExecutionPosition>,state:&mut LogicState, logics:&HashMap<usize,LogicResource>) {

        while !stack.is_empty() {
            let stack_pos = stack.len()-1;
            let entry = stack[stack_pos];
            let logic_sequence = logics[&entry.get_logic()].get_logic_sequence();
            let actions = logic_sequence.get_operations();
            let mut exec = entry;
            loop {
                match logic_sequence.interpret_instructions(resources,state,&exec,actions) {
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
        self.keys.push(key_code);
    }
    
    pub fn clear_keys(&mut self) {
        self.keys.clear();
    }

    pub fn run(&mut self) {

        let mut resuming = !self.stack.is_empty();
        let mutable_state = &mut self.state;
        let mutable_stack = &mut self.stack;

        // delay
        // clear keybuffer
        mutable_state.clear_keys();

        mutable_state.set_flag(&FLAG_COMMAND_ENTERED, false);
        mutable_state.set_flag(&FLAG_SAID_ACCEPTED_INPUT, false);
        // poll keyb/joystick
        for k in &self.keys {
            if (*k as u32) <256 {
                mutable_state.key_pressed(*k as u8);
            }
        }

        if !resuming {
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
            mutable_state.set_var(&VAR_OBJ_EDGE, 0);
            mutable_state.set_var(&VAR_OBJ_TOUCHED_BORDER, 0);
            mutable_state.set_flag(&FLAG_ROOM_FIRST_TIME, false);
            mutable_state.set_flag(&FLAG_RESTART_GAME, false);
            mutable_state.set_flag(&FLAG_RESTORE_GAME, false);
            // update all controlled objects on screen
            // if new room issued, rerun logic
            if mutable_state.get_new_room()!=0 {
                LogicSequence::new_room(mutable_state,mutable_state.get_new_room());
            } else {
                break;
            }
        }

        render_sprites(&self.resources,mutable_state,false);
    }
}
