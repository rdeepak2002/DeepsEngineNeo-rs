use glow::*;
use sdl2::EventPump;

pub(crate) trait Renderer {
    fn update(&self);
    fn swap_buffers(&self);
    fn destroy(&self);
    fn should_close(&self) -> bool;
}

pub(crate) struct OpenGLRenderer {
    gl: glow::Context,
    program: glow::NativeProgram,
    vertex_array: glow::NativeVertexArray,
    #[cfg(feature = "sdl2")]
    window: sdl2::video::Window,
    events_loop: EventPump,
}

impl OpenGLRenderer {
    pub fn new() -> Self {
        unsafe {
            // Create a context from a WebGL2 context on wasm32 targets
            #[cfg(target_arch = "wasm32")]
            let (gl, shader_version) = {
                use wasm_bindgen::JsCast;
                let canvas = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("canvas")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .unwrap();
                let webgl2_context = canvas
                    .get_context("webgl2")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::WebGl2RenderingContext>()
                    .unwrap();
                let gl = glow::Context::from_webgl2_context(webgl2_context);
                (gl, "#version 300 es")
            };

            // Create a context from a sdl2 window
            #[cfg(feature = "sdl2")]
            let (gl, shader_version, window, mut events_loop, _context) = {
                let sdl = sdl2::init().unwrap();
                let video = sdl.video().unwrap();
                let gl_attr = video.gl_attr();
                gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
                gl_attr.set_context_version(3, 0);
                let window = video
                    .window("Hello triangle!", 1024, 769)
                    .opengl()
                    .resizable()
                    .build()
                    .unwrap();
                let gl_context = window.gl_create_context().unwrap();
                let gl = glow::Context::from_loader_function(|s| {
                    video.gl_get_proc_address(s) as *const _
                });
                let event_loop = sdl.event_pump().unwrap();
                (gl, "#version 330", window, event_loop, gl_context)
            };

            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vertex_array));

            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
                r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            gl.use_program(Some(program));
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            #[cfg(feature = "sdl2")]
            Self {
                gl,
                program,
                vertex_array,
                window,
                events_loop,
            }

            // #[cfg(target_arch = "wasm32")]
            // Self { gl, program, vertex_array, window }
        }
    }

    pub fn update(&mut self) {
        println!("test 2");

        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    pub fn swap_buffers(&mut self) {
        unsafe {
            #[cfg(feature = "sdl2")]
            {
                self.window.gl_swap_window();
            }
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vertex_array);
        }
    }

    pub fn should_close(&mut self) -> bool {
        #[cfg(feature = "sdl2")]
        {
            for event in self.events_loop.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => return true,
                    _ => {}
                }
            }
        }

        return false;
    }
}
