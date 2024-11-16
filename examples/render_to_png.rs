use std::{
    fs::File,
    io::BufWriter,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use ul_next::{
    config::Config,
    platform::{self, LogLevel, Logger},
    renderer::Renderer,
    view::ViewConfig,
    Library,
};

struct MyLogger;

impl Logger for MyLogger {
    fn log_message(&mut self, log_level: LogLevel, message: String) {
        println!("{:?}: {}", log_level, message);
    }
}

/// This example is ported from the `Ultralight` main repository.

///  In this sample we'll load a string of HTML and render it to a PNG.
///  
///  Since we're rendering offscreen and don't need to create any windows or handle any user input,
///  we won't be using [`App::new`] and will instead be using the Ultralight API directly with our
///  own custom main loop.
///
///  Our main loop waits for the page to finish loading by listening to
///  event from `on_finished_loading` and then renders the page to a PNG.
fn main() {
    let lib = Library::linked();

    // create our config
    // we are using the defaults, but we can change that if we need
    let config = Config::start().build(lib.clone()).unwrap();

    // Since we're not using App::Create(), we must provide our own Platform API handlers.
    //
    // The Platform API handlers we can set are:
    //
    //  - platform::set_logger          (empty, optional, supported)
    //  - platform::set_gpu_driver      (empty, optional, not supported)
    //  - platform::set_font_loader     (empty, **required**, not supported)
    //  - platform::set_filesystem     (empty, optional, not supported)
    //  - platform::set_clipboard       (empty, optional, supported)
    //  - platform::set_surface_factory (defaults to BitmapSurfaceFactory, **required**, not supported)
    //
    // The only Platform API handler we are required to provide is a font loader.
    // we can't use a custom font loader yet, but we can use the default one.
    platform::enable_platform_fontloader(lib.clone());

    // use the default filesystem and we specify the root path, which
    // all `file:///` URLs will be resolved against.
    //
    // NOTE: custom filesystem is still not supported in this library
    platform::enable_platform_filesystem(lib.clone(), "./examples").unwrap();

    // Register a logger that logs messages to the console.
    //
    // We can use [`platform::enable_default_logger`] and provide a log file
    // to log to it.
    platform::set_logger(lib.clone(), MyLogger);

    // Create our Renderer (you should only create this once per application).
    //
    // The Renderer singleton maintains the lifetime of the library and is required before
    // creating any Views. It should outlive any Views.
    //
    // You should set up the Platform methods before creating this.
    let renderer = Renderer::create(config).unwrap();

    // Create our View.
    //
    // Views are sized containers for loading and displaying web content.
    //
    // Our view config uses 2x DPI scale and "Arial" as the default font.
    //
    // We make sure GPU acceleration is disabled so we can render to an offscreen bitmap.
    let view_config = ViewConfig::start()
        .initial_device_scale(2.0)
        .font_family_standard("Arial")
        .is_accelerated(false)
        .build(lib.clone())
        .unwrap();

    // We use the default session by passing `None`.
    let view = renderer
        .create_view(1600, 1600, &view_config, None)
        .unwrap();

    // we setup a listener to listen to event until the loading is finished
    let done = Rc::new(AtomicBool::new(false));
    let done_clone = done.clone();
    view.set_finish_loading_callback(move |_view, _frame_id, is_main_frame, _url| {
        if is_main_frame {
            done_clone.store(true, Ordering::SeqCst);
        }
    });

    // Load a string of HTML into our View. (For code readability, the string is defined in the
    // HTML_STRING const at the bottom of this file)
    //
    // @note:
    //   This operation may not complete immediately-- we will call [`Renderer::update`] continuously
    //   and wait for the `finish_loading` event before rendering our View.
    //
    // Views can also load remote URLs, try replacing the code below with:
    //    view.load_url("https://en.wikipedia.org");
    view.load_html(HTML_STRING).unwrap();

    println!("starting main loop");
    // running the main loop and waiting until the loading is finished
    while !done.load(Ordering::SeqCst) {
        // Continuously update until OnFinishLoading() is called below (which sets done = true).
        //
        // @note:
        //   Calling Renderer::Update handles any pending network requests, resource loads, and
        //   JavaScript timers.
        renderer.update();
        thread::sleep(Duration::from_millis(10));
    }
    println!("finished main loop");

    // Render our View.
    //
    // @note:
    //   Calling Renderer::Render will render any dirty Views to their respective Surfaces.
    renderer.render();

    // Get the Surface for our View.
    // This will be `None` if the View is accelerated.
    let mut surface = view.surface().unwrap();

    let width = surface.width();
    let height = surface.height();
    let bytes_per_pixel = surface.row_bytes() / width;
    // RGBA
    assert!(bytes_per_pixel == 4);

    // Get the raw pixels of the surface
    let pixels = surface.lock_pixels().unwrap();

    // TODO: add support for the library's `Bitmap::write_png` method.
    println!("writing PNG file (this could take some time)");
    // Save the surface to a PNG file.
    {
        let path = Path::new(r"./result.png");
        let file = File::create(path).unwrap();
        let w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&pixels).unwrap(); // Save
    }

    println!("Saved result.png");
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
