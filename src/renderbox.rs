use std::fs::File;
use std::io::BufReader;

use glium::winit::{event::WindowEvent, window::Window};
use glium::{Surface, Texture2d};

use crate::window::{ApplicationContext, GliumDisplay};

pub struct RenderboxState {
    pub opengl_texture: Texture2d,
    pub dest_texture: Texture2d,
}

// a png file for GIMP splash screen
const IMAGE_PATH: &str = "/usr/share/gimp/3.0/images/gimp-splash.png";

impl ApplicationContext for RenderboxState {
    fn new<'a>(display: &GliumDisplay) -> Self {
        //       let mut data = Vec::new();
        let file = File::open(IMAGE_PATH).expect(&format!("cannot open file: {}", IMAGE_PATH));
        let reader = BufReader::new(file);
        //      let _ = file.read(&mut data);

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

        Self {
            opengl_texture,
            dest_texture,
        }
    }

    fn update(&mut self) {}
    fn draw_frame(&mut self, display: &GliumDisplay) {
        let frame = display.draw();
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

        self.dest_texture
            .as_surface()
            .fill(&frame, glium::uniforms::MagnifySamplerFilter::Linear);
        frame.finish().unwrap();
    }
    fn handle_window_event(&mut self, _ev: &WindowEvent, _window: &Window) {}
}
