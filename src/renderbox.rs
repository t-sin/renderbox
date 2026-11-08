use std::sync::mpsc::Receiver;
use std::time::Instant;

use glium::index::NoIndices;
use glium::winit::{event::WindowEvent, window::Window};
use glium::{Program, Surface, Texture2d, VertexBuffer};
use notify::EventKind;
use notify::event::ModifyKind;

use crate::shader::{VERTEX_SHADER_CODE, compile_shaders};
use crate::{
    shader,
    window::{ApplicationContext, GliumDisplay},
};

pub struct RenderboxOption {
    pub fshader_path: String,
    pub file_event_receiver: Option<Receiver<Result<notify::Event, notify::Error>>>,
}

#[derive(Clone, Copy)]
struct CardVertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

const CARD_VERTECES: [CardVertex; 6] = [
    CardVertex {
        position: [-1.0, -1.0],
        tex_coord: [0.0, 0.0],
    },
    CardVertex {
        position: [1.0, -1.0],
        tex_coord: [1.0, 0.0],
    },
    CardVertex {
        position: [-1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    CardVertex {
        position: [-1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    CardVertex {
        position: [1.0, -1.0],
        tex_coord: [1.0, 0.0],
    },
    CardVertex {
        position: [1.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
];
glium::implement_vertex!(CardVertex, position location(0), tex_coord location(1));

#[derive(Default)]
struct Mouse {
    x: f32,
    y: f32,
}

pub struct RenderboxState {
    screen_texture: Texture2d,
    new_fshader: Option<String>,
    program: Program,
    card_vbo: VertexBuffer<CardVertex>,
    indices: NoIndices,
    start_at: Instant,
    mouse: Mouse,
}

impl ApplicationContext<RenderboxOption> for RenderboxState {
    fn new(option: &RenderboxOption, display: &GliumDisplay) -> Self {
        let fshader_src = shader::read_text_file(&option.fshader_path).expect("cannot read file");
        let program = shader::compile_shaders(display, shader::VERTEX_SHADER_CODE, &fshader_src)
            .expect("cannot compile shader");

        let resolution = display.get_framebuffer_dimensions();
        let screen_texture = glium::Texture2d::empty_with_format(
            display,
            glium::texture::UncompressedFloatFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            resolution.0,
            resolution.1,
        )
        .unwrap();
        screen_texture
            .as_surface()
            .clear_color_srgb(1.0, 1.0, 1.0, 1.0);

        let card_vbo = VertexBuffer::new(display, &CARD_VERTECES).unwrap();
        let indices = NoIndices(glium::index::PrimitiveType::TrianglesList);

        let start_at = Instant::now();
        let mouse = Mouse::default();

        let new_fshader = None;

        Self {
            screen_texture,
            new_fshader,
            program,
            card_vbo,
            indices,
            start_at,
            mouse,
        }
    }

    fn update(&mut self, option: &RenderboxOption) {
        if let Some(rx) = option.file_event_receiver.as_ref() {
            let mut is_updated = false;

            for result in rx.try_iter() {
                match result {
                    Ok(ev) => {
                        if let EventKind::Modify(ModifyKind::Data(_)) = ev.kind {
                            is_updated |= true;
                        }
                    }
                    Err(err) => {
                        println!("notify::Error = {:?}", err);
                    }
                }
            }

            if is_updated {
                println!("file update detected!!");
                let fshader =
                    shader::read_text_file(&option.fshader_path).expect("cannot read file");
                println!("new frag shader has read");
                self.new_fshader = Some(fshader);
            }
        }
    }
    fn draw_frame(&mut self, _option: &RenderboxOption, display: &GliumDisplay) {
        if let Some(fshader) = self.new_fshader.as_ref() {
            match compile_shaders(display, VERTEX_SHADER_CODE, fshader) {
                Ok(program) => {
                    self.program = program;
                }
                Err(err) => {
                    println!("===== shader compile error! =====");
                    println!("{:?}", err);
                    println!("");
                    self.new_fshader = None;
                }
            }
        }

        let mut frame = display.draw();

        let time = self.start_at.elapsed().as_secs_f32();
        let resolution = display.get_framebuffer_dimensions();
        let resolution = (resolution.0 as f32, resolution.1 as f32);
        let mouse = (self.mouse.x, self.mouse.y);
        let uniforms = glium::uniform! {
            time: time,
            resolution: resolution,
            mouse: mouse,
        };
        //println!("res, t, mouse = {:?}, {:?}, {:?}", resolution, time, mouse);

        frame
            .draw(
                &self.card_vbo,
                &self.indices,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();

        frame.finish().unwrap();
    }

    fn handle_window_event(
        &mut self,
        _option: &RenderboxOption,
        ev: &WindowEvent,
        _window: &Window,
    ) {
        if let WindowEvent::CursorMoved { position, .. } = ev {
            self.mouse.x = position.x as f32;
            self.mouse.y = position.y as f32;
        }
    }
}
