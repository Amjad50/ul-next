use bindgen::{Bindings, Builder};
use std::path::{Path, PathBuf};

const INCLUDE_DIR: &str = "-Iultralight_api";

const APPCORE_FUNCS: [&str; 60] = [
    "ulAppGetMainMonitor",
    "ulAppGetRenderer",
    "ulAppIsRunning",
    "ulAppQuit",
    "ulAppRun",
    "ulAppSetUpdateCallback",
    "ulCreateApp",
    "ulCreateOverlay",
    "ulCreateOverlayWithView",
    "ulCreateSettings",
    "ulCreateWindow",
    "ulDestroyApp",
    "ulDestroyOverlay",
    "ulDestroySettings",
    "ulDestroyWindow",
    "ulEnableDefaultLogger",
    "ulEnablePlatformFileSystem",
    "ulEnablePlatformFontLoader",
    "ulMonitorGetHeight",
    "ulMonitorGetScale",
    "ulMonitorGetWidth",
    "ulOverlayFocus",
    "ulOverlayGetHeight",
    "ulOverlayGetView",
    "ulOverlayGetWidth",
    "ulOverlayGetX",
    "ulOverlayGetY",
    "ulOverlayHasFocus",
    "ulOverlayHide",
    "ulOverlayIsHidden",
    "ulOverlayMoveTo",
    "ulOverlayResize",
    "ulOverlayShow",
    "ulOverlayUnfocus",
    "ulSettingsSetAppName",
    "ulSettingsSetDeveloperName",
    "ulSettingsSetFileSystemPath",
    "ulSettingsSetForceCPURenderer",
    "ulSettingsSetLoadShadersFromFileSystem",
    "ulWindowClose",
    "ulWindowGetHeight",
    "ulWindowGetNativeHandle",
    "ulWindowGetPositionX",
    "ulWindowGetPositionY",
    "ulWindowGetScale",
    "ulWindowGetScreenHeight",
    "ulWindowGetScreenWidth",
    "ulWindowGetWidth",
    "ulWindowHide",
    "ulWindowIsFullscreen",
    "ulWindowIsVisible",
    "ulWindowMoveTo",
    "ulWindowMoveToCenter",
    "ulWindowPixelsToScreen",
    "ulWindowScreenToPixels",
    "ulWindowSetCloseCallback",
    "ulWindowSetCursor",
    "ulWindowSetResizeCallback",
    "ulWindowSetTitle",
    "ulWindowShow",
];

fn common_builder() -> Builder {
    Builder::default()
        .derive_debug(true)
        .derive_partialeq(true)
        .generate_inline_functions(true)
        .derive_default(true)
        .clang_arg(INCLUDE_DIR)
        .blocklist_function("ulPlatformSetFontLoader")
}

fn post_process_loaded_and_write<P: AsRef<Path>, const IS_APPCORE: bool>(
    bindings: Bindings,
    out_path: P,
) {
    let buf = bindings.to_string();

    // put `#[cfg(feature = "loaded")]` for related contents
    let content = buf
        .replace(
            "__library:",
            r#"
        #[cfg(feature = "loaded")]
        __library:"#,
        )
        .replace(
            "pub unsafe fn new",
            r#"
            #[cfg(feature = "loaded")]
            #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
            pub unsafe fn load_from"#,
        )
        .replace(
            "pub unsafe fn from_library",
            r#"
            #[cfg(feature = "loaded")]
            #[cfg_attr(docsrs, doc(cfg(feature = "loaded")))]
            unsafe fn from_library"#,
        )
        .replace(
            "::libloading::Library,",
            "Option<::std::sync::Arc<::libloading::Library>>,",
        )
        .replace(
            "__library,",
            "__library: Some(::std::sync::Arc::new(__library)),",
        )
        .replace("pub struct ", "#[derive(Clone)]\npub struct ");

    let func_regex = regex::Regex::new(r"pub (\w+):").unwrap();
    let all_funcs: Vec<_> = func_regex
        .captures_iter(&content)
        .map(|m| m.get(1).unwrap().as_str())
        .collect();

    let mut middle_fields = String::new();

    for func in all_funcs.iter() {
        middle_fields.push_str(&format!("{}: crate::linked::{},\n", func, func));
    }

    let appcore_pre = if IS_APPCORE { "appcore_" } else { "" };

    let linked_func = format!(
        r#"
            #[cfg(feature = "{appcore_pre}linked")]
            #[cfg_attr(docsrs, doc(cfg(feature = "{appcore_pre}linked")))]
            pub const fn linked() -> Self {{
                Self {{
                    #[cfg(feature = "loaded")]
                    __library: None,
                    {middle_fields}
                }}
            }}
        "#,
    );

    // put this function at the top of the `impl` block
    let impl_regex = regex::Regex::new(r"impl.*\{").unwrap();
    let content = impl_regex.replace(&content, |caps: &regex::Captures| {
        format!("{}\n{}", &caps[0], linked_func)
    });

    {
        std::fs::write(&out_path, content.as_bytes()).expect("Couldn't write bindings!");
    }

    // run rustfmt
    let rustfmt_bin = std::env::var("RUSTFMT").unwrap_or_else(|_| "rustfmt".to_string());
    let status = std::process::Command::new(rustfmt_bin)
        .arg(out_path.as_ref())
        .status()
        .expect("Failed to run rustfmt");
    if !status.success() {
        panic!("rustfmt failed");
    }
}

fn defines() {
    let header_path = "ultralight_api/AppCore/CAPI.h";

    let bindings = common_builder()
        .header(header_path)
        .allowlist_var("^UL.*|kJS.*|JS.*|ul.*|WK.*")
        .allowlist_type("^UL.*|kJS.*|JS.*|ul.*|WK.*")
        // this is `static extern` variable, we don't want others to use it
        .blocklist_var("kJSClassDefinitionEmpty")
        .ignore_functions()
        .ignore_methods()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::current_dir().unwrap())
        .join("src")
        .join("defines.rs");

    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

fn ultralight_linked() {
    let header_path = "ultralight_api/Ultralight/CAPI.h";

    let bindings = common_builder()
        .header(header_path)
        .allowlist_recursively(false)
        .allowlist_function("^UL.*|JS.*|ul.*|WK.*")
        .raw_line("use crate::defines::*;")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::current_dir().unwrap())
        .join("src")
        .join("linked")
        .join("ultralight.rs");

    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

fn appcore_linked() {
    let header_path = "ultralight_api/AppCore/CAPI.h";

    let mut builder = common_builder()
        .header(header_path)
        .allowlist_recursively(false)
        .raw_line("use crate::defines::*;");

    for func in APPCORE_FUNCS.iter() {
        builder = builder.allowlist_function(func);
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::current_dir().unwrap())
        .join("src")
        .join("linked")
        .join("appcore.rs");

    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

fn library_ultralight() {
    let header_path = "ultralight_api/Ultralight/CAPI.h";

    // Configure bindgen with options similar to the command-line version
    let bindings = common_builder()
        .header(header_path)
        .allowlist_function("^UL.*|JS.*|ul.*|WK.*")
        .allowlist_recursively(false)
        .dynamic_library_name("Ultralight")
        .dynamic_link_require_all(true)
        .raw_line("use crate::defines::*;")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::current_dir().unwrap())
        .join("src")
        .join("library")
        .join("ultralight.rs");

    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

    post_process_loaded_and_write::<_, false>(bindings, out_path);
}

fn library_appcore() {
    let header_path = "ultralight_api/AppCore/CAPI.h";

    let mut builder = common_builder()
        .header(header_path)
        .allowlist_recursively(false)
        .dynamic_library_name("AppCore")
        .dynamic_link_require_all(true)
        .raw_line("use crate::defines::*;");

    for func in APPCORE_FUNCS.iter() {
        builder = builder.allowlist_function(func);
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::current_dir().unwrap())
        .join("src")
        .join("library")
        .join("appcore.rs");

    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

    post_process_loaded_and_write::<_, true>(bindings, out_path);
}

fn linked() {
    println!("Generating Ultralight linked bindings");
    ultralight_linked();

    println!("Generating AppCore linked bindings");
    appcore_linked();
}

fn libraries() {
    println!("Generating Ultralight bindings");
    library_ultralight();

    println!("Generating AppCore bindings");
    library_appcore();
}

fn main() {
    println!("Generating: defines.rs");
    defines();

    println!("Generating: linked/{{appcore,ultralight}}.rs");
    linked();

    println!("Generating: library/{{appcore,ultralight}}.rs");
    libraries();
}
