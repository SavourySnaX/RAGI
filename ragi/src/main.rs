
use std::time::Duration;
use glow::HasContext;
use helpers::{conv_rgba, double_pic_width, conv_rgba_transparent};
use interpretter::{Interpretter, LogicExecutionPosition, AgiKeyCodes, get_cells_clamped, pri_slice_for_baseline, VAR_CURRENT_ROOM, SCREEN_WIDTH_USIZE, SCREEN_HEIGHT_USIZE};
use logic::*;


use picture::{PIC_HEIGHT_USIZE, PIC_WIDTH_USIZE};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use imgui::*;


const XMAS:bool=false;
const DP1:bool=false;
const KQ1:bool=false;
const KQ2:bool=false;
const KQ3:bool=false;
const KQ4:bool=false;
const LL1:bool=false;
const SQ1:bool=true;
const SQ2:bool=false;
const SQ2_F:bool=false;
const GR:bool =false;
const BC:bool =false;
const MH1:bool = false;
const TMP1:bool = false;

fn main() -> Result<(), String> {

    let mut interpretter:Interpretter;

    if XMAS {
        interpretter=Interpretter::new("../images/AGI-XMAS/","2.272").unwrap();
        //interpretter.set_breakpoint(4,44,true);
    } else if DP1 {
        interpretter=Interpretter::new("../images/agi_demo_pack_1/","2.915").unwrap();
        //interpretter.set_breakpoint(161,1,true);
        interpretter.set_breakpoint_on_instruction(&ActionOperation::ClearTextRect((type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),)), false);
    } else if KQ1 {
        //interpretter=Interpretter::new("../images/King's Quest v2.0F (AGI 2.425) (1987)(Sierra On-Line, Inc.) [Adventure]/","2.425").unwrap();
        interpretter=Interpretter::new("../images/King's Quest v1.0U (1986)(Sierra On-Line, Inc.) [Adventure][!]/","2.272").unwrap();
        //interpretter.breakpoints.insert(LogicExecutionPosition::new(53,145), false);
        //interpretter.breakpoints.insert(LogicExecutionPosition::new(53,233), false);
        //interpretter.breakpoints.insert(LogicExecutionPosition::new(53,181), false);
        //interpretter.breakpoints.insert(LogicExecutionPosition::new(53,251), false);
    } else if KQ2 {
        interpretter=Interpretter::new("../images/King's Quest II- Romancing the Throne v2.1 (1987)(Sierra On-Line, Inc.) [Adventure]/","2.411").unwrap();

    } else if KQ3 {
        interpretter=Interpretter::new("../images/King's Quest III- To Heir is Human v2.14 (1988)(Sierra On-Line, Inc.) [Adventure]/","2.936").unwrap();
    } else if KQ4 {
        interpretter=Interpretter::new("../images/King's Quest IV- The Perils of Rosella v2.0 (AGI Engine) (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.086").unwrap();
    } else if LL1 {
        interpretter=Interpretter::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/","2.440").unwrap();
        //interpretter.breakpoints.insert(LogicExecutionPosition::new(2,151), false);
        //interpretter.breakpoints.insert(LogicExecutionPosition::new(3,151), false);
        //interpretter.breakpoints.insert(LogicExecutionPosition::new(6,151), false);
        //interpretter.set_breakpoint_on_instruction(&ActionOperation::Draw((type_object_from_u8(1),)), false);
        //interpretter.set_breakpoint_on_instruction(&ActionOperation::AddToPic((type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),type_num_from_u8(0),)), false);
        //interpretter.set_breakpoint_on_instruction(&ActionOperation::MenuInput(()), false);
        //interpretter.set_breakpoint(0,153,true);

        //cheat bypass questions
        //interpretter.state.set_flag(&TypeFlag::from(110),true);
    } else if SQ1 {
        interpretter=Interpretter::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/","2.089").unwrap();
        //interpretter.set_breakpoint(5,54,false);
        //interpretter.set_breakpoint_on_instruction(&ActionOperation::Draw((type_object_from_u8(1),)), false);
    } else if SQ2 {
        interpretter=Interpretter::new("../images/Space Quest II- Chapter II - Vohaul's Revenge v2.0C (1987)(Sierra On-Line, Inc.) [Adventure]/","2.917").unwrap();
        interpretter.set_breakpoint(2,147,true);
    } else if SQ2_F {
        interpretter=Interpretter::new("../images/Space Quest II V2.0F/","2.936").unwrap();
        //interpretter.set_breakpoint(6,126,true);
    } else if GR {
        interpretter=Interpretter::new("../images/Gold Rush! v2.01 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.149").unwrap();
        interpretter.set_breakpoint(1,1,true);
    } else if BC {
        interpretter=Interpretter::new("../images/Black Cauldron, The v2.10 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.098").unwrap();
        interpretter.set_breakpoint(0,0,true);
    } else if MH1 {
        interpretter=Interpretter::new("../images/Manhunter- New York v1.22 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.102").unwrap();
        //interpretter.set_breakpoint(153,150,true);
    } else if TMP1 {
        interpretter=Interpretter::new("../images/TexMcPhilip1QuestForThePapacy/","3.002.102").unwrap();
    } else {
        panic!("NO GAME SET");
    }


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

    let textures_ui = TexturesUi::new(&gl,&mut textures,10+256*2);

    let live_debug_view=false;
    let mut debug_texture_index:usize=0;
    let mut resume=false;
    let mut step=false;
    let mut pause=false;
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
                Event::KeyDown { keycode: Some(code), keymod:modifier,..} => {
                    if let Some(agi_code) = map_keycodes(code,modifier) {
                        interpretter.key_code_pressed(agi_code);
                    }
                }
                _ => {}
            }
        }

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        // The rest of the game loop goes here...
        let mut just_paused=false;
        if !pause {
            interpretter.run(resume,step);
            just_paused=interpretter.is_paused() || step;
            pause=just_paused;
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

        Window::new("BUTTONS").build(&ui, || {
            if pause {
                resume = ui.button("Resume");
                step = ui.button("Step");
            } else {
                if ui.button("Pause") {
                    pause=true;
                    just_paused=true;
                    //insert a temporary breakpoint on the current room
                    interpretter.breakpoints.insert(LogicExecutionPosition::new(interpretter.state.get_var(&VAR_CURRENT_ROOM).into(),0),true);
                    //step=true;
                }
            }
        });

        pause = pause && !(resume||step);
        Window::new("OBJECTS").build(&ui, || {
            for index in interpretter.state.active_objects_indices_sorted_pri_y() {
                let obj_num = &TypeObject::from(index as u8);
                let obj=interpretter.state.object(obj_num);
                let visible = obj.get_visible();
                let priority = obj.get_priority();
                TreeNode::new(format!("Object {} : Priority {}",index,priority)).flags(if visible {TreeNodeFlags::BULLET} else {TreeNodeFlags::OPEN_ON_ARROW}).build(&ui, || {
                    let mut c = usize::from(obj.get_cel());
                    let cels = get_cells_clamped(&interpretter.resources, &obj);
                    if c>=cels.len() {
                        c=cels.len()-1;
                    }
                    let cell = &cels[c];
                    let dbl = double_pic_width(cell.get_data());
                    let rgb=conv_rgba_transparent(&dbl, cell.get_transparent_colour());
                    let width = cell.get_width() as usize;
                    let width = width * 2;
                    let height = cell.get_height() as _;
                    textures_ui.update(&gl, index*2+10, width, height, &rgb);
                    Image::new(textures_ui.get_generated_texture(index*2+10),[width as f32,height as f32]).build(&ui);
                    // And we the priority pixels under this sprites base line please
                    let x=obj.get_x() as usize;
                    let y=obj.get_y() as usize;
                    let w = width/2;
                    let pri_slice = pri_slice_for_baseline(&interpretter.state, x, y, w);
                    let rgba = conv_rgba(pri_slice);
                    textures_ui.update(&gl, index*2+1+10, w,1,&rgba);
                    Image::new(textures_ui.get_generated_texture(index*2+1+10),[(w as f32)*16.0,16.0]).build(&ui);
                    ui.text_wrapped(format!("{:?}",obj));
                });
            }
        });

        Window::new("BG_DEBUG").build(&ui, || {
            ui.combo_simple_string("Texture", &mut debug_texture_index, &["picture_buffer","priority_buffer","back_buffer","text_buffer","gfx_buffer"]);

            match debug_texture_index {
                0 => {
                    let d = double_pic_width(interpretter.state.picture());
                    let d = conv_rgba(&d);
                    textures_ui.update(&gl, 1, PIC_WIDTH_USIZE*2, PIC_HEIGHT_USIZE, &d);
                    Image::new(textures_ui.get_generated_texture(1),[(PIC_WIDTH_USIZE*2) as f32,PIC_HEIGHT_USIZE as f32]).build(&ui);
                },
                1 => {
                    let d = double_pic_width(interpretter.state.priority());
                    let d = conv_rgba(&d);
                    textures_ui.update(&gl, 1, PIC_WIDTH_USIZE*2, PIC_HEIGHT_USIZE, &d);
                    Image::new(textures_ui.get_generated_texture(1),[(PIC_WIDTH_USIZE*2) as f32,PIC_HEIGHT_USIZE as f32]).build(&ui);
                },
                2 => {
                    let d = conv_rgba(interpretter.state.back_buffer());
                    textures_ui.update(&gl, 2, SCREEN_WIDTH_USIZE, SCREEN_HEIGHT_USIZE, &d);
                    Image::new(textures_ui.get_generated_texture(2),[SCREEN_WIDTH_USIZE as f32,SCREEN_HEIGHT_USIZE as f32]).build(&ui);
                },
                3 => {
                    let d = conv_rgba(interpretter.state.text_buffer());
                    textures_ui.update(&gl, 2, SCREEN_WIDTH_USIZE, SCREEN_HEIGHT_USIZE, &d);
                    Image::new(textures_ui.get_generated_texture(2),[SCREEN_WIDTH_USIZE as f32,SCREEN_HEIGHT_USIZE as f32]).build(&ui);
                },
                4 => {
                    let d = conv_rgba(interpretter.state.screen_buffer());
                    textures_ui.update(&gl, 2, SCREEN_WIDTH_USIZE, SCREEN_HEIGHT_USIZE, &d);
                    Image::new(textures_ui.get_generated_texture(2),[SCREEN_WIDTH_USIZE as f32,SCREEN_HEIGHT_USIZE as f32]).build(&ui);
                },
                _ => {},
            }
        });

        Window::new("LOGIC").build(&ui, || {
            if (live_debug_view || pause) && !interpretter.state.stack.is_empty() {
                let top_of_stack = &interpretter.state.stack[interpretter.state.stack.len()-1];
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
                                if Selectable::new(format!("{} {}",if *top_of_stack==address {">"} else {" "},address)).flags(SelectableFlags::SPAN_ALL_COLUMNS).build_with_ref(&ui,&mut selected) {
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
            if live_debug_view || pause {
                for (index,f) in interpretter.state.get_flags().enumerate() {
                    if f {
                        ui.text(format!("{:3} : {}", index, f));
                    }
                }
            }
        });

        Window::new("VARS").build(&ui, || {
            if live_debug_view || pause {
                for (index,v) in interpretter.state.get_vars().enumerate() {
                    if v!=0 {
                        ui.text(format!("{:3} : {}", index, v));
                    }
                }
            }
        });
        
        Window::new("STRINGS").build(&ui, || {
            if live_debug_view || pause {
                for (index,s) in interpretter.state.get_strings().enumerate() {
                    if !s.is_empty() {
                        ui.text(format!("{:3} : {}", index, s));
                    }
                }
            }
        });


        Window::new("STACK").build(&ui, || {
            if live_debug_view || pause {
                for a in (&interpretter.state.stack).into_iter().rev() {
                    ui.text(format!("Logic : {} | PC : {}", a.get_logic(),a.get_pc()));
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


pub fn map_keycodes(code:Keycode,modifiers:Mod) -> Option<AgiKeyCodes> {
    if modifiers == Mod::LALTMOD || modifiers == Mod::RALTMOD {
        match code {
            Keycode::A => Some(AgiKeyCodes::AltA),
            Keycode::B => Some(AgiKeyCodes::AltB),
            Keycode::C => Some(AgiKeyCodes::AltC),
            Keycode::D => Some(AgiKeyCodes::AltD),
            Keycode::E => Some(AgiKeyCodes::AltE),
            Keycode::F => Some(AgiKeyCodes::AltF),
            Keycode::G => Some(AgiKeyCodes::AltG),
            Keycode::H => Some(AgiKeyCodes::AltH),
            Keycode::I => Some(AgiKeyCodes::AltI),
            Keycode::J => Some(AgiKeyCodes::AltJ),
            Keycode::K => Some(AgiKeyCodes::AltK),
            Keycode::L => Some(AgiKeyCodes::AltL),
            Keycode::M => Some(AgiKeyCodes::AltM),
            Keycode::N => Some(AgiKeyCodes::AltN),
            Keycode::O => Some(AgiKeyCodes::AltO),
            Keycode::P => Some(AgiKeyCodes::AltP),
            Keycode::Q => Some(AgiKeyCodes::AltQ),
            Keycode::R => Some(AgiKeyCodes::AltR),
            Keycode::S => Some(AgiKeyCodes::AltS),
            Keycode::T => Some(AgiKeyCodes::AltT),
            Keycode::U => Some(AgiKeyCodes::AltU),
            Keycode::V => Some(AgiKeyCodes::AltV),
            Keycode::W => Some(AgiKeyCodes::AltW),
            Keycode::X => Some(AgiKeyCodes::AltX),
            Keycode::Y => Some(AgiKeyCodes::AltY),
            Keycode::Z => Some(AgiKeyCodes::AltZ),
            _ => None,
        }
    } else if modifiers == Mod::LCTRLMOD || modifiers == Mod::RCTRLMOD {
        match code {
            Keycode::A => Some(AgiKeyCodes::CtrlA),
            Keycode::B => Some(AgiKeyCodes::CtrlB),
            Keycode::C => Some(AgiKeyCodes::CtrlC),
            Keycode::D => Some(AgiKeyCodes::CtrlD),
            Keycode::E => Some(AgiKeyCodes::CtrlE),
            Keycode::F => Some(AgiKeyCodes::CtrlF),
            Keycode::G => Some(AgiKeyCodes::CtrlG),
            Keycode::H => Some(AgiKeyCodes::CtrlH),
            Keycode::I => Some(AgiKeyCodes::CtrlI),
            Keycode::J => Some(AgiKeyCodes::CtrlJ),
            Keycode::K => Some(AgiKeyCodes::CtrlK),
            Keycode::L => Some(AgiKeyCodes::CtrlL),
            Keycode::M => Some(AgiKeyCodes::CtrlM),
            Keycode::N => Some(AgiKeyCodes::CtrlN),
            Keycode::O => Some(AgiKeyCodes::CtrlO),
            Keycode::P => Some(AgiKeyCodes::CtrlP),
            Keycode::Q => Some(AgiKeyCodes::CtrlQ),
            Keycode::R => Some(AgiKeyCodes::CtrlR),
            Keycode::S => Some(AgiKeyCodes::CtrlS),
            Keycode::T => Some(AgiKeyCodes::CtrlT),
            Keycode::U => Some(AgiKeyCodes::CtrlU),
            Keycode::V => Some(AgiKeyCodes::CtrlV),
            Keycode::W => Some(AgiKeyCodes::CtrlW),
            Keycode::X => Some(AgiKeyCodes::CtrlX),
            Keycode::Y => Some(AgiKeyCodes::CtrlY),
            Keycode::Z => Some(AgiKeyCodes::CtrlZ),
            _ => None,
        }
    } else {
        match code {
            Keycode::Left => Some(AgiKeyCodes::Left),
            Keycode::Right => Some(AgiKeyCodes::Right),
            Keycode::Down => Some(AgiKeyCodes::Down),
            Keycode::Up => Some(AgiKeyCodes::Up),
            Keycode::Escape => Some(AgiKeyCodes::Escape),
            Keycode::Return => Some(AgiKeyCodes::Enter),
            Keycode::Space => Some(AgiKeyCodes::Space),
            Keycode::Backspace => Some(AgiKeyCodes::Backspace),
            Keycode::Tab => Some(AgiKeyCodes::TAB),
            Keycode::Num0 => Some(AgiKeyCodes::_0),
            Keycode::Num1 => Some(AgiKeyCodes::_1),
            Keycode::Num2 => Some(AgiKeyCodes::_2),
            Keycode::Num3 => Some(AgiKeyCodes::_3),
            Keycode::Num4 => Some(AgiKeyCodes::_4),
            Keycode::Num5 => Some(AgiKeyCodes::_5),
            Keycode::Num6 => Some(AgiKeyCodes::_6),
            Keycode::Num7 => Some(AgiKeyCodes::_7),
            Keycode::Num8 => Some(AgiKeyCodes::_8),
            Keycode::Num9 => Some(AgiKeyCodes::_9),
            Keycode::A => Some(AgiKeyCodes::A),
            Keycode::B => Some(AgiKeyCodes::B),
            Keycode::C => Some(AgiKeyCodes::C),
            Keycode::D => Some(AgiKeyCodes::D),
            Keycode::E => Some(AgiKeyCodes::E),
            Keycode::F => Some(AgiKeyCodes::F),
            Keycode::G => Some(AgiKeyCodes::G),
            Keycode::H => Some(AgiKeyCodes::H),
            Keycode::I => Some(AgiKeyCodes::I),
            Keycode::J => Some(AgiKeyCodes::J),
            Keycode::K => Some(AgiKeyCodes::K),
            Keycode::L => Some(AgiKeyCodes::L),
            Keycode::M => Some(AgiKeyCodes::M),
            Keycode::N => Some(AgiKeyCodes::N),
            Keycode::O => Some(AgiKeyCodes::O),
            Keycode::P => Some(AgiKeyCodes::P),
            Keycode::Q => Some(AgiKeyCodes::Q),
            Keycode::R => Some(AgiKeyCodes::R),
            Keycode::S => Some(AgiKeyCodes::S),
            Keycode::T => Some(AgiKeyCodes::T),
            Keycode::U => Some(AgiKeyCodes::U),
            Keycode::V => Some(AgiKeyCodes::V),
            Keycode::W => Some(AgiKeyCodes::W),
            Keycode::X => Some(AgiKeyCodes::X),
            Keycode::Y => Some(AgiKeyCodes::Y),
            Keycode::Z => Some(AgiKeyCodes::Z),
            Keycode::F1 => Some(AgiKeyCodes::F1),
            Keycode::F2 => Some(AgiKeyCodes::F2),
            Keycode::F3 => Some(AgiKeyCodes::F3),
            Keycode::F4 => Some(AgiKeyCodes::F4),
            Keycode::F5 => Some(AgiKeyCodes::F5),
            Keycode::F6 => Some(AgiKeyCodes::F6),
            Keycode::F7 => Some(AgiKeyCodes::F7),
            Keycode::F8 => Some(AgiKeyCodes::F8),
            Keycode::F9 => Some(AgiKeyCodes::F9),
            Keycode::F10 => Some(AgiKeyCodes::F10),
            Keycode::Minus => Some(AgiKeyCodes::Minus),
            Keycode::Equals => Some(AgiKeyCodes::Equals),
            _ => {/* println!("Unmapped Keycode {} : {}",code as i32,scode as i32);*/ None},
        }
    }
}

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
