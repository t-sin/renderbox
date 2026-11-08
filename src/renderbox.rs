use std::time::Instant;

use glium::index::NoIndices;
use glium::program::ProgramCreationError;
use glium::winit::{event::WindowEvent, window::Window};
use glium::{Program, Surface, Texture2d, VertexBuffer};

use crate::window::{ApplicationContext, GliumDisplay};

const OPENGL_SHADER_HEADER_STRING: &str = r#"#version 460"#;
const VERTEX_SHADER_CODE: &str = r#"#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;
out vec2 frag_coord;

void main() {
//  frag_coord = tex_coord;
  gl_Position = vec4(position, 0.0, 1.0);
}
"#;
const FRAGMENT_SHADER_CODE: &str = r#"#version 460
precision highp float;

out vec4 frag_color;

float PI = 3.14159265;

uniform float time;
//uniform vec2 mouse;
uniform vec2 resolution;

float dt(vec2 p, vec2 c, float r) {
  vec2 dp = p - c;
  float d = sqrt(pow(dp.x, 2.0) + pow(dp.y, 2.0));

  float d0 = (d - r);
  float d1 = (r - d);
  float dout = pow(d0, 0.9243) * 0.021;
  float din = pow(d1, 0.912) * 0.02226;
  return (d >= r) ? dout : din;
}

void main(void) {
  vec2 pf_orig = gl_FragCoord.xy - (resolution.xy / 2.0);
  vec2 pf = pf_orig * 2.0;
  vec2 pc = vec2(0.0, 0.0);
  //vec2 mouse = resolution.xy - resolution.xy / 2.0;

  vec2 pr = pf + vec2(27.124 * cos(pf.x / 58.8 + PI * time * 0.9), 16.3456 * sin(pf.y / 40.128 + time * 0.54));
  float r = pow(dt(pr, pc, 435.0), 0.04);

  vec2 pg = pf + vec2(19.0234 * cos(pf.x / 81.0 + PI * time * 0.835), 12.3456 * sin(pf.y / 111.5 + time * 0.984));
  float g = pow(dt(pg, pc, 463.0), 0.312);

  vec2 pb = pf + vec2(31.124 * cos(pf.x / 113.0 + PI * time * 2.3), 40.342 * sin(pf.y / 86.135 + time * 1.54));
  float b = pow(dt(pb, pc, 494.0), 0.4231);

  frag_color = vec4(r, g, b, 1.0);
}
"#;

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

pub struct RenderboxOption {
    pub image_path: String,
}

pub struct RenderboxState {
    screen_texture: Texture2d,
    program: Program,
    card_vbo: VertexBuffer<CardVertex>,
    indices: NoIndices,
    start_at: Instant,
}

fn compile_shaders(
    display: &GliumDisplay,
    vshader_src: &str,
    fshader_src: &str,
) -> Result<Program, ()> {
    let res = Program::from_source(display, vshader_src, fshader_src, None);
    if let Err(err) = res {
        match err {
            ProgramCreationError::CompilationError(message, shader_type) => {
                println!("shader compilation error!");
                println!("    shader type = {:?}", shader_type);
                println!("{}", message);
            }
            ProgramCreationError::LinkingError(message) => {
                println!("shader linking error!");
                println!("{}", message);
            }
            err => println!("program creation error: {:?}", err),
        }

        return Err(());
    }

    return Ok(res.unwrap());
}

impl ApplicationContext<RenderboxOption> for RenderboxState {
    fn new<'a>(option: &RenderboxOption, display: &GliumDisplay) -> Self {
        let program = compile_shaders(display, VERTEX_SHADER_CODE, FRAGMENT_SHADER_CODE)
            .expect("cannot compile shaders");

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

        Self {
            screen_texture,
            program,
            card_vbo,
            indices,
            start_at,
        }
    }

    fn update(&mut self) {}
    fn draw_frame(&mut self, display: &GliumDisplay) {
        let mut frame = display.draw();

        let sampler = self
            .screen_texture
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear);
        //frame.clear_color(0.0, 0.0, 0.0, 1.0);

        let resolution = display.get_framebuffer_dimensions();
        let resolution = (resolution.0 as f32, resolution.1 as f32);
        let time = self.start_at.elapsed().as_secs_f32();
        let uniforms = glium::uniform! {
            tex: sampler,
            time: time,
            resolution: resolution,
        };

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
    fn handle_window_event(&mut self, _ev: &WindowEvent, _window: &Window) {}
}
