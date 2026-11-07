use std::fs::File;
use std::io::BufReader;

use glium::index::NoIndices;
use glium::winit::{event::WindowEvent, window::Window};
use glium::{Program, Surface, Texture2d, VertexBuffer};

use crate::window::{ApplicationContext, GliumDisplay};

const VERTEX_SHADER_CODE: &str = r#"#version 460
out vec2 v_tex_coords;

const vec2 positions[] = vec2[](
  vec2(-1.0, -1.0), vec2( 1.0, -1.0), vec2(-1.0,  1.0),
  vec2(-1.0,  1.0), vec2( 1.0, -1.0), vec2( 1.0,  1.0)
);
const vec2 tex_coords[] = vec2[](
  vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
  vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0)
);

void main() {
  v_tex_coords = tex_coords[gl_VertexID];
  gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
}
"#;
const FRAGMENT_SHADER_CODE: &str = r#"#version 460

in vec2 v_tex_coords;
out vec4 color;

layout(binding = 0) uniform sampler2D tex;

void main() {
  vec4 tex_color = texture(tex, v_tex_coords);
  color = tex_color * 0.1;
}"#;

#[derive(Clone, Copy)]
struct EmptyVertex {
    _dummy: u8,
}
glium::implement_vertex!(EmptyVertex, _dummy);

pub struct RenderboxOption {
    pub image_path: String,
}

pub struct RenderboxState {
    pub opengl_texture: Texture2d,
    pub dest_texture: Texture2d,
    program: Program,
    dummy_vbo: VertexBuffer<EmptyVertex>,
    indices: NoIndices,
}

impl ApplicationContext<RenderboxOption> for RenderboxState {
    fn new<'a>(option: &RenderboxOption, display: &GliumDisplay) -> Self {
        let file = File::open(&option.image_path)
            .expect(&format!("cannot open file: {}", option.image_path));
        let reader = BufReader::new(file);

        let program = Program::from_source(display, VERTEX_SHADER_CODE, FRAGMENT_SHADER_CODE, None)
            .expect("cannot compile shaders");

        let image = image::load(reader, image::ImageFormat::Png)
            .unwrap()
            .to_rgba8();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let opengl_texture = glium::Texture2d::new(display, image).unwrap();

        // building a 1024x1024 empty texture
        let dest_texture = glium::Texture2d::empty_with_format(
            display,
            glium::texture::UncompressedFloatFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            1024,
            1024,
        )
        .unwrap();
        dest_texture
            .as_surface()
            .clear_color_srgb(0.0, 0.0, 1.0, 1.0);

        let dummy_vertices = vec![EmptyVertex { _dummy: 0 }; 6];
        let dummy_vbo = VertexBuffer::new(display, &dummy_vertices).unwrap();
        let indices = NoIndices(glium::index::PrimitiveType::TrianglesList);

        Self {
            opengl_texture,
            dest_texture,
            program,
            dummy_vbo,
            indices,
        }
    }

    fn update(&mut self) {}
    fn draw_frame(&mut self, display: &GliumDisplay) {
        let mut frame = display.draw();
        let dest_rect = glium::BlitTarget {
            left: 0,
            bottom: 0,
            width: self.opengl_texture.get_width().cast_signed(),
            height: self
                .opengl_texture
                .get_height()
                .expect("height is none!?!?")
                .cast_signed(),
        };

        self.opengl_texture.as_surface().blit_whole_color_to(
            &self.dest_texture.as_surface(),
            &dest_rect,
            glium::uniforms::MagnifySamplerFilter::Linear,
        );

        let sampler = self
            .dest_texture
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear);
        let uniforms = glium::uniform! {tex: sampler};

        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame
            .draw(
                &self.dummy_vbo,
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
