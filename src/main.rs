use std::io::Cursor;
use std::num::NonZeroU32;

use glium::winit::event::WindowEvent;
use glium::{Display, Surface, glutin};
use glium::{
    Texture2d,
    glutin::{
        context::NotCurrentGlContext,
        display::{GetGlDisplay, GlDisplay},
        surface::WindowSurface,
    },
};
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowAttributes},
};

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 640;

trait AppContext {
    fn update(&mut self);
    fn draw_frame(&mut self, display: &Display<WindowSurface>);
    fn handle_window_event(&mut self, ev: &WindowEvent, window: &Window);
}

struct RenderboxState {
    opengl_texture: Texture2d,
    dest_texture: Texture2d,
}

struct GliumState {
    window: Window,
    display: Display<WindowSurface>,
    context: RenderboxState,
}

#[derive(Default)]
struct RenderboxApp {
    state: Option<GliumState>,
}

impl RenderboxState {
    fn new<'a>(display: &Display<WindowSurface>) -> Self {
        let image = image::load(
            // a png file for GIMP splash screen
            Cursor::new(&include_bytes!("/usr/share/gimp/3.0/images/gimp-splash.png")[..]),
            image::ImageFormat::Png,
        )
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
}

impl AppContext for RenderboxState {
    fn update(&mut self) {}
    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
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

impl ApplicationHandler for RenderboxApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.state = Some(GliumState::new(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("closing window because of request...");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.state.as_ref() {
                    state.display.resize(new_size.into());
                }
            }

            glium::winit::event::WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.context.update();
                    state.context.draw_frame(&state.display);
                }
            }
            glium::winit::event::WindowEvent::KeyboardInput {
                event:
                    glium::winit::event::KeyEvent {
                        state: glium::winit::event::ElementState::Pressed,
                        logical_key:
                            glium::winit::keyboard::Key::Named(glium::winit::keyboard::NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            // Every other event
            ev => {
                if let Some(state) = self.state.as_mut() {
                    state.context.handle_window_event(&ev, &state.window);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

impl GliumState {
    fn new<'a>(event_loop: &ActiveEventLoop) -> Self {
        let wa = WindowAttributes::default()
            .with_resizable(true)
            .with_blur(true)
            .with_title("Renderbox");

        // cf. https://github.com/glium/glium/blob/1776d155fe45c0951d9564b3f822283daaf053a1/examples/support/mod.rs#L177
        let config_template_builder = glutin::config::ConfigTemplateBuilder::new();
        let display_builder = glutin_winit::DisplayBuilder::new().with_window_attributes(Some(wa));

        // First we create a window
        let (window, gl_config) = display_builder
            .build(event_loop, config_template_builder, |mut configs| {
                // Just use the first configuration since we don't have any special preferences here
                configs.next().unwrap()
            })
            .unwrap();
        let window = window.unwrap();

        // Then the configuration which decides which OpenGL version we'll end up using, here we just use the default which is currently 3.3 core
        // When this fails we'll try and create an ES context, this is mainly used on mobile devices or various ARM SBC's
        // If you depend on features available in modern OpenGL Versions you need to request a specific, modern, version. Otherwise things will very likely fail.
        let window_handle = window
            .window_handle()
            .expect("couldn't obtain window handle");
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(Some(window_handle.into()));
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(Some(window_handle.into()));

        let not_current_gl_context = Some(unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        });

        // Determine our framebuffer size based on the window size, or default to 800x600 if it's invisible
        let (width, height): (u32, u32) = (WINDOW_WIDTH, WINDOW_HEIGHT);
        let attrs = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window_handle.into(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
        // Now we can create our surface, use it to make our context current and finally create our display
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        let current_context = not_current_gl_context
            .unwrap()
            .make_current(&surface)
            .unwrap();
        let display = glium::Display::from_context_surface(current_context, surface).unwrap();

        let context = RenderboxState::new(&display);
        GliumState {
            window: window,
            display: display,
            context: context,
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut renderbox_app = RenderboxApp::default();
    let _ = event_loop.run_app(&mut renderbox_app);
}
