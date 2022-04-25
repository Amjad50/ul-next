use std::{env, path::PathBuf, rc::Rc};

use rust_ul_next::{
    app::{App, Settings},
    view::View,
    window::WindowFlags,
};

fn main() {
    // builds relative path from the exe location to the current location
    // The library doesn't support absolute paths
    let mut path = PathBuf::from_iter(env::current_exe().unwrap().components().map(|_| ".."))
        .to_string_lossy()
        .to_string();
    path.push_str(&env::current_dir().unwrap().to_string_lossy());

    // set the file system path to the current location, to access resources
    let app = App::new(
        Some(Settings::start().file_system_path(&path).build()),
        None,
    );

    let window = app.create_window(
        1280,
        720,
        false,
        WindowFlags {
            borderless: false,
            titled: true,
            resizable: true,
            maximizable: true,
            hidden: false,
        },
    );

    let overlay = window.create_overlay(window.width(), window.height(), 0, 0);

    overlay.view().set_add_console_message_callback(
        |_view, _message_source, message_level, message, _line, _column, _source_id| {
            println!("{:?}: {}", message_level, message);
        },
    );

    overlay.view().load_html(
        r#"
        <html>
            <head>
                <style>
                    body {
                        background-color: black;
                        color: white;
                        font-size: 100px;
                    }
                </style>
            </head>
            <body>Hello</body>
            <script>
                console.log("Hello from JavaScript!");
            </script>
        </html>"#,
    );

    let finished = |_view: &View, _frame_id, _is_main_frame, _url| println!("loaded!");
    let dom_ready = |_view: &View, _frame_id, _is_main_frame, _url| println!("dom ready!");

    overlay.view().set_finish_loading_callback(finished);
    overlay.view().set_dom_ready_callback(dom_ready);

    window.set_resize_callback(move |_window, width, height| {
        overlay.resize(width, height);
    });

    let app = Rc::new(app);
    let app_clone = app.clone();
    window.set_close_callback(move |_window| {
        app_clone.quit();
    });

    app.run();
}
