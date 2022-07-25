
use glow::HasContext;
use helpers::{conv_rgba, double_pic_width, conv_rgba_transparent};
use logic::{LogicResource, LogicSequence, LogicState, LogicExecutionPosition, GameResources, render_sprites, update_sprites, VAR_OBJ_TOUCHED_BORDER, VAR_OBJ_EDGE, FLAG_SAID_ACCEPTED_INPUT, FLAG_COMMAND_ENTERED, FLAG_ROOM_FIRST_TIME, FLAG_RESTART_GAME, FLAG_RESTORE_GAME, VAR_CURRENT_ROOM, get_cells, VAR_EGO_MOTION_DIR, OBJECT_EGO, TypeObject, get_direction_from_delta};


use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use imgui::*;
use std::collections::HashMap;
use std::time::Duration;

struct TexturesUi {
    generated_textures: Vec<TextureId>,
    gl_textures: Vec<u32>,
}

impl TexturesUi {
    fn new(gl: &glow::Context, textures: &mut Textures<glow::Texture>,num:usize) -> Self {
        let mut generated_textures:Vec<TextureId> = Vec::new();
        let mut gl_textures:Vec<u32> = Vec::new();
        generated_textures.reserve(num);
        gl_textures.reserve(num);
        for _ in 0..num {
            let (generated_texture,gl_texture) = Self::generate(gl, textures);
            generated_textures.push(generated_texture);
            gl_textures.push(gl_texture);
        }
        Self {
            generated_textures,gl_textures
        }
    }

    fn get_generated_texture(&self,index:usize) -> TextureId {
        self.generated_textures[index]
    }

    pub fn update(&self,gl:&glow::Context,index:usize, width:usize,height:usize,data:&[u8]) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.gl_textures[index]));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as _, // When generating a texture like this, you're probably working in linear color space
                width as _,
                height as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            )
        }
    }

    /// Generate dummy 1x1 texture with sane settings - Will be overwritten by gui later
    fn generate(
        gl: &glow::Context,
        textures: &mut Textures<glow::Texture>,
    ) -> (TextureId,u32) {
        let mut data = Vec::with_capacity(1 * 1);
        for i in 0..1 {
            for j in 0..1 {
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
                glow::NEAREST as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as _,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as _, // When generating a texture like this, you're probably working in linear color space
                1 as _,
                1 as _,
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

    let mut interpretter=Interpretter::new("../images/King's Quest v1.0U (1986)(Sierra On-Line, Inc.) [Adventure][!]/","2.272").unwrap();
    //let mut interpretter=Interpretter::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/").unwrap(); let version="2.440";
    //let mut interpretter=Interpretter::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/","2.089").unwrap();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);

    let window = video_subsystem.window("R.A.G.I", 640*2+400, 400*2)
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

    let mut renderer = imgui_glow_renderer::Renderer::initialize(&gl,&mut imgui,&mut textures, false)
        .expect("failed to create renderer");
    let mut event_pump = sdl_context.event_pump()?;

    let textures_ui = TexturesUi::new(&gl,&mut textures,64);

    interpretter.breakpoints.insert(LogicExecutionPosition::new(2,0), false);

    let mut resume=false;
    let mut step=false;
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
        let mut just_paused=false;
        if !interpretter.is_paused() || resume || step {
            interpretter.run(resume,step);
            just_paused=interpretter.is_paused();
        }

        resume=false;
        step=false;

        // imgui windows etc
        let pic = conv_rgba(interpretter.state.final_buffer());

        textures_ui.update(&gl,0, 320, 200, &pic);
        
        let ui = imgui.frame();

        Window::new("MAIN GAME").resizable(false).build(&ui, || {
            Image::new(textures_ui.get_generated_texture(0),[640.0,400.0]).build(&ui);
        });

        Window::new("OBJECTS").build(&ui, || {
            for index in interpretter.state.active_objects_indices() {
                let obj_num = &TypeObject::from(index as u8);
                let obj=interpretter.state.object(obj_num);
                let visible = obj.get_visible();
                TreeNode::new(format!("Object {}",index)).flags(if visible {TreeNodeFlags::BULLET} else {TreeNodeFlags::OPEN_ON_ARROW}).build(&ui, || {
                    let c = usize::from(obj.get_cel());
                    let cels = get_cells(&interpretter.resources, &obj);
                    let cell = &cels[c];
                    let dbl = double_pic_width(cell.get_data());
                    let rgb=conv_rgba_transparent(&dbl, cell.get_transparent_colour());
                    let width = cell.get_width() as usize;
                    let width = width * 2;
                    let height = cell.get_height() as _;
                    textures_ui.update(&gl, index+1, width, height, &rgb);
                    Image::new(textures_ui.get_generated_texture(index+1),[width as f32,height as f32]).build(&ui);
                    ui.text_wrapped(format!("{:?}",obj));
                });
            }
        });

        Window::new("LOGIC").build(&ui, || {
            if interpretter.is_paused() {
                let top_of_stack = &interpretter.stack[interpretter.stack.len()-1];
                let file = top_of_stack.get_logic();
                let logic = interpretter.resources.logic.get(&file);
                if !logic.is_none() {
                    if let Some(_t) = ui.begin_table_with_flags("logic_table",2,TableFlags::RESIZABLE|TableFlags::SCROLL_Y|TableFlags::SCROLL_X|TableFlags::NO_KEEP_COLUMNS_VISIBLE) {
                        for (g,s) in logic.unwrap().get_disassembly_iterator(&interpretter.resources.words, &interpretter.resources.objects) {
                            ui.table_next_row();
                            ui.table_set_column_index(0);
                            if let Some(g) = g {
                                let address =LogicExecutionPosition::new(file,g.into());
                                let mut selected = interpretter.breakpoints.contains_key(&address);
                                if Selectable::new(format!("{} {:?}",if *top_of_stack==address {">"} else {" "},address)).flags(SelectableFlags::SPAN_ALL_COLUMNS).build_with_ref(&ui,&mut selected) {
                                    if interpretter.breakpoints.contains_key(&address) {
                                        interpretter.breakpoints.remove(&address);
                                    } else {
                                        interpretter.breakpoints.insert(address, false);
                                    }
                                }
                                if just_paused && *top_of_stack == address {
                                    ui.set_scroll_here_y();
                                }
                            }
                            ui.table_set_column_index(1);
                            ui.text(s);
                        }
                    }
                }
            }
        });

        Window::new("FLAGS").build(&ui, || {
            if interpretter.is_paused() {
                for (index,f) in interpretter.state.get_flags().enumerate() {
                    if f {
                        ui.text(format!("{:3} : {}", index, f));
                    }
                }
            }
        });

        Window::new("VARS").build(&ui, || {
            if interpretter.is_paused() {
                for (index,v) in interpretter.state.get_vars().enumerate() {
                    if v!=0 {
                        ui.text(format!("{:3} : {}", index, v));
                    }
                }
            }
        });
        
        Window::new("STRINGS").build(&ui, || {
            if interpretter.is_paused() {
                for (index,s) in interpretter.state.get_strings().enumerate() {
                    if !s.is_empty() {
                        ui.text(format!("{:3} : {}", index, s));
                    }
                }
            }
        });


        Window::new("STACK").build(&ui, || {
            if interpretter.is_paused() {
                for a in (&interpretter.stack).into_iter().rev() {
                    ui.text(format!("Logic : {} | PC : {}", a.get_logic(),a.get_pc()));
                }
            }
        });

        Window::new("BUTTONS").build(&ui, || {
            if interpretter.is_paused() {
                resume = ui.button("Resume");
                step = ui.button("Step");
            } else {
                if ui.button("Pause") {
                    //insert a temporary breakpoint on the current room
                    interpretter.breakpoints.insert(LogicExecutionPosition::new(interpretter.state.get_var(&VAR_CURRENT_ROOM).into(),0),true);
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
    pub breakpoints:HashMap<LogicExecutionPosition,bool>,
}

impl Interpretter {
    pub fn new(base_path:&'static str,version:&str) -> Result<Interpretter,String> {
        let resources = GameResources::new(base_path,version)?;
        Ok(Interpretter {
            resources,
            state: LogicState::new(),
            stack: Vec::new(),
            keys: Vec::new(),
            breakpoints: HashMap::new(),
        })
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
                match logic_sequence.interpret_instructions(resources,state,&exec,actions) {
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
        Self::do_call(breakpoints, resources, stack, state,logics,resume,single_step);
    }

    pub fn key_code_pressed(&mut self,key_code:Keycode) {
        self.keys.push(key_code);
    }
    
    pub fn clear_keys(&mut self) {
        self.keys.clear();
    }

    pub fn run(&mut self,resume:bool,single_step:bool) {

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
            if mutable_state.is_ego_player_controlled() {

                let mut dx=0;
                let mut dy=0;
                for k in &self.keys {
                    match k {
                        Keycode::Left => dx=-1,
                        Keycode::Right => dx=1,
                        Keycode::Up => dy=-1,
                        Keycode::Down => dy=1,
                        _ => {},
                    }
                }

                mutable_state.set_var(&VAR_EGO_MOTION_DIR, get_direction_from_delta(dx, dy));
            } else {
                let d = mutable_state.get_var(&VAR_EGO_MOTION_DIR);
                mutable_state.mut_object(&OBJECT_EGO).set_direction(d);
            }
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
            
            Self::call(&mut self.breakpoints,&self.resources,mutable_stack,mutable_state, 0, &self.resources.logic,resume,single_step);
            if !mutable_stack.is_empty() {
                return;
            } else {
                resuming=false;
            }

            // dir of EGO <- var(6)
            let d = mutable_state.get_var(&VAR_EGO_MOTION_DIR);
            mutable_state.mut_object(&OBJECT_EGO).set_direction(d);
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
