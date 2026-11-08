use glium::{Program, program::ProgramCreationError};

use crate::window::GliumDisplay;

pub fn compile_shaders(
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

pub const OPENGL_SHADER_HEADER_STRING: &str = r#"#version 460"#;

pub const VERTEX_SHADER_CODE: &str = r#"#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;
out vec2 frag_coord;

void main() {
//  frag_coord = tex_coord;
  gl_Position = vec4(position, 0.0, 1.0);
}
"#;

pub const FRAGMENT_SHADER_CODE: &str = r#"#version 460
precision highp float;

//in vec4 gl_FragCoord     // a position of this fragment (origin: bottom-left, unit: pixels)
out vec4 frag_color;       // a color output of this fragment

uniform float time;        // seconds from start Renderbox
uniform vec2 resolution;   // resolution of drawing area
uniform vec2 mouse;        // position of mouse cursor (origin: top-left, unit: pixels)
//uniform vec2 sound;        // a value of stereo audio input
//uniform ??? spectrum       // a spectrum values of audio input

void main(void) {
  vec2 p = gl_FragCoord.xy - (resolution.xy / 2.0);
  vec2 c = vec2(mouse.x, resolution.y - mouse.y) - (resolution.xy / 2.0);

  float d = sqrt(pow(p.x - c.x, 2.0) + pow(p.y - c.y, 2.0));
  float v = abs(d - 100) < 5 ? 1.0 : 0.0;

  frag_color = vec4(v, v, v, 1.0);
}
"#;
