use glium::glutin::dpi::PhysicalSize;
use glium::{glutin, implement_vertex};
use glium::{
    glutin::{event_loop::EventLoop, window::WindowBuilder, ContextBuilder},
    uniform, Display, Surface,
};
use glium::{index::PrimitiveType, program::ProgramCreationInput, Program};
use rust_ul_next::{config::Config, platform, renderer::Renderer, view::ViewConfig};

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new().with_inner_size(PhysicalSize::new(900, 600));
    let cb = ContextBuilder::new().with_srgb(false);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let config = Config::start().build();

    // basic setup (check `render_to_png` for full explanation)
    platform::enable_platform_fontloader();
    platform::enable_platform_file_system(".");
    platform::enable_default_logger("./log.log");

    // use `glium` gpu driver, which is included in the library under the
    // feature `glium`
    let (sender, mut receiver) = rust_ul_next::gpu_driver::glium::create_gpu_driver(&display);
    platform::set_gpu_driver(sender);

    let renderer = Renderer::create(config);

    let view_config = ViewConfig::start()
        .initial_device_scale(1.0)
        .is_accelerated(true)
        .build();

    let view = renderer.create_view(900, 600, &view_config, None);

    view.load_html(HTML_STRING);

    // create vertex/index buffers and program which will be used
    // to blit the `rendered` texture from the GPU driver to the window.
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(
            &display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 1.0],
                },
            ],
        )
        .unwrap()
    };

    // building the index buffer
    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
            .unwrap();

    let program = Program::new(
        &display,
        ProgramCreationInput::SourceCode {
            vertex_shader: "
                #version 140
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            fragment_shader: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;
                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            ",
            transform_feedback_varyings: None,
            // there is a bug in glium, the default back buffer
            // is `srgb`, even though we set `with_srgb` to `false`
            // in the context builder. But it doesn't work, so we manually modify
            // the program to output `linear` color.
            outputs_srgb: true,
            uses_point_size: false,
        },
    )
    .unwrap();

    let mut update_and_draw = move |size: Option<(u32, u32)>| {
        if let Some(size) = size {
            view.resize(size.0, size.1);
        }

        renderer.update();

        // in case of resize of the view needs repaint, render and blit
        // otherwise, exit.
        if !(view.needs_paint() || size.is_some()) {
            return;
        }

        // painting
        renderer.render();
        // flush the drawing commands
        receiver.render();

        let render_target = view.render_target().unwrap();

        let texture = receiver.get_texture(&render_target.texture_id).unwrap();

        let uniforms = uniform! {
            tex: texture.sampled()
        };

        let mut target = display.draw();
        target.clear_color_srgb(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    };

    update_and_draw(None);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                // Redraw the triangle when the window is resized.
                glutin::event::WindowEvent::Resized(size) => {
                    update_and_draw(Some((size.width, size.height)));

                    glutin::event_loop::ControlFlow::Poll
                }
                _ => glutin::event_loop::ControlFlow::Poll,
            },
            _ => {
                update_and_draw(None);
                glutin::event_loop::ControlFlow::Poll
            }
        };
    });
}

const HTML_STRING: &str = r#"
<html>
  <head>
    <style type="text/css">
      body {
        margin: 0;
        padding: 0;
        overflow: hidden;
        color: black;
        font-family: Arial;
        background: linear-gradient(-45deg, #acb4ff, #f5d4e2);
        display: flex;
        justify-content: center;
        align-items: center;
      }
      div {
        width: 350px;
        height: 350px;
        text-align: center;
        border-radius: 25px;
        background: linear-gradient(-45deg, #e5eaf9, #f9eaf6);
        box-shadow: 0 7px 18px -6px #8f8ae1;
      }
      h1 {
        padding: 1em;
      }
      p {
        background: white;
        padding: 2em;
        margin: 40px;
        border-radius: 25px;
      }
    </style>
  </head>
  <body>
    <div>
      <h1>Hello World!</h1>
      <p>Welcome to Ultralight!</p>
    </div>
  </body>
</html>"#;
