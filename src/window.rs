use std::{marker::PhantomData, num::NonZeroU32};

use glium::glutin::{
    context::NotCurrentGlContext,
    display::{GetGlDisplay, GlDisplay},
    surface::WindowSurface,
};
use glium::winit::event::WindowEvent;
use glium::{Display, glutin};
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowAttributes},
};

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub type GliumDisplay = Display<WindowSurface>;

pub trait ApplicationContext<T> {
    fn new(option: &T, display: &GliumDisplay) -> Self;
    fn update(&mut self);
    fn draw_frame(&mut self, display: &GliumDisplay);
    fn handle_window_event(&mut self, ev: &WindowEvent, window: &Window);
}

pub struct GliumState<T, U>
where
    T: ApplicationContext<U>,
{
    pub window: Window,
    pub display: GliumDisplay,
    pub context: T,
    option: PhantomData<U>,
}

#[derive(Default)]
pub struct Application<T, U>
where
    T: ApplicationContext<U>,
{
    pub glium_state: Option<GliumState<T, U>>,
    pub option: U,
}

impl<T, U> GliumState<T, U>
where
    T: ApplicationContext<U>,
{
    fn new<'a>(option: &U, event_loop: &ActiveEventLoop) -> Self {
        let wa = WindowAttributes::default()
            .with_resizable(true)
            .with_blur(true)
            .with_title(crate::WINDOW_TITLE);

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

        let context = T::new(option, &display);
        GliumState {
            window: window,
            display: display,
            context: context,
            option: PhantomData,
        }
    }
}

impl<T, U> ApplicationHandler for Application<T, U>
where
    T: ApplicationContext<U>,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.glium_state = Some(GliumState::<T, U>::new(&self.option, event_loop));
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
                if let Some(state) = self.glium_state.as_ref() {
                    state.display.resize(new_size.into());
                }
            }

            glium::winit::event::WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.glium_state {
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
                if let Some(state) = self.glium_state.as_mut() {
                    state.context.handle_window_event(&ev, &state.window);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.glium_state {
            state.window.request_redraw();
        }
    }
}

impl<T, U> Application<T, U>
where
    T: ApplicationContext<U>,
{
    fn new(option: U) -> Self {
        Self {
            glium_state: None,
            option: option,
        }
    }

    pub fn start_app(option: U) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut renderbox_app = Application::<T, U>::new(option);
        let _ = event_loop.run_app(&mut renderbox_app);
    }
}
