use std::rc::Rc;

use ul_next::{
    app::App,
    javascript::{JSObject, JSPropertyAttributes, JSValue},
    platform,
    window::WindowFlags,
    Library,
};

fn main() {
    let lib = Library::linked();

    platform::enable_platform_filesystem(lib.clone(), "./examples").unwrap();

    // use default settings and configs
    let app = App::new(lib.clone(), None, None).unwrap();

    // create window
    let window = app
        .create_window(
            900,
            600,
            false,
            WindowFlags {
                borderless: false,
                titled: true,
                resizable: true,
                maximizable: true,
                hidden: false,
            },
        )
        .unwrap();

    window.set_title("Javascript example");

    // Create a web-content overlay that spans the entire window.
    //
    // You can create multiple overlays per window, each overlay has its own View which can be
    // used to load and display web-content.
    //
    // AppCore automatically manages focus, keyboard/mouse input, and GPU painting for each active
    // overlay. Dropping the overlay will remove it from the window.
    let overlay = window
        .create_overlay(window.width(), window.height(), 0, 0)
        .unwrap();

    overlay.view().set_dom_ready_callback(move |view, _, _, _| {
        let ctx = view.lock_js_context();
        let global = ctx.global_object();

        let func = JSObject::new_function_with_callback(&ctx, |ctx, _this, _args| {
            // call the javascript function `JavascriptCallback`
            // can be done by running a script
            println!(
                "Javascript returned {:?}",
                ctx.evaluate_script("JavascriptCallback();")
                    .unwrap()
                    .as_string()
                    .unwrap()
            );
            // or by getting the object and calling it
            println!(
                "Javascript returned {:?}",
                ctx.global_object()
                    .get_property("JavascriptCallback")
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .call_as_function(None, &[])
                    .unwrap()
                    .as_string()
                    .unwrap()
            );

            Ok(JSValue::new_string(ctx, "And Hello from Rust!<br>"))
        });

        global
            .set_property("GetRustMessage", &func, JSPropertyAttributes::default())
            .unwrap();
    });

    // Load a string of HTML into our overlay's View
    overlay.view().load_html(HTML_STRING).unwrap();

    // create a clone from the app so that we can use it in the window close callback
    let app = Rc::new(app);
    let app_clone = app.clone();

    // add window close event callback to exit the application
    window.set_close_callback(move |_window| {
        app_clone.quit();
    });

    // run the main loop
    app.run();
}

const HTML_STRING: &str = r#"
<html>
  <head>
    <style type="text/css">
      * { -webkit-user-select: none; }
      body {
        font-family: -apple-system, 'Segoe UI', Ubuntu, Arial, sans-serif;
        text-align: center;
        background: linear-gradient(#FFF, #DDD);
        padding: 2em;
      }
      body.rainbow {
        background: linear-gradient(90deg, #ff2363, #fff175, #68ff9d,
                                           #45dce0, #6c6eff, #9e23ff, #ff3091);
        background-size: 1000% 1000%;
        animation: ScrollGradient 10s ease infinite;
      }
      @keyframes ScrollGradient {
        0%   { background-position:0% 50%; }
        50%  { background-position:100% 50%; }
        100% { background-position:0% 50%; }
      }
      #message {
        padding-top: 2em;
        color: white;
        font-weight: bold;
        font-size: 24px;
        text-shadow: 1px 1px rgba(0, 0, 0, 0.4);
      }
    </style>
    <script type="text/javascript">
    function HandleButton(evt) {
      // Call our Rust callback 'GetRustMessage'
      var message = GetRustMessage();

      // Display the result in our 'message' div element and apply the
      // rainbow effect to our document's body.
      document.getElementById('message').innerHTML += message;
      document.body.classList.add('rainbow');
    }
    function JavascriptCallback() {
        // This function is called from Rust
        document.getElementById('message').innerHTML += "Hello from Javascript callback!<br>";

        return "Hello from Javascript!";
    }
    </script>
  </head>
  <body>
    <button onclick="HandleButton(event);">Get the Secret Message!</button>
    <div id="message"></div>
  </body>
</html>"#;
