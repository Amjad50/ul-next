use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    // skip if we are building doc
    #[cfg(feature = "docs_only")]
    {
        // use return, to reduce the indentation level
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ultralight_dir = out_dir.join("Ultralight");

    println!("cargo:rerun-if-changed=build.rs");

    if ultralight_dir.is_dir() {
        fs::remove_dir_all(&ultralight_dir)
            .expect("Could not remove already existing Ultralight repo");
    }

    let git_status = Command::new("git")
        .args(["clone", "https://github.com/ultralight-ux/Ultralight"])
        .current_dir(&out_dir)
        .status()
        .expect("Git is needed to retrieve the ultralight C++ library!");

    assert!(git_status.success(), "Couldn't clone Ultralight library");

    let git_status = Command::new("git")
        .args([
            "reset",
            "--hard",
            "208d653e872b29234bbd4a5fef6dec403f3dfdbd",
        ])
        .current_dir(&ultralight_dir)
        .status()
        .expect("Git is needed to retrieve the ultralight C++ library!");

    assert!(
        git_status.success(),
        "Could not reset git head to desired revision"
    );

    let dst = cmake::build(ultralight_dir.join("packager"));
    let lib_bin_dir = dst.join("bin");

    if cfg!(feature = "only-ul-deps") {
        let allowed_files = [
            "Ultralight",
            "UltralightCore",
            "WebCore",
            "AppCore",
            "gstreamer-full-1.0",
        ];
        for entry in fs::read_dir(&lib_bin_dir).unwrap().flatten() {
            let path = entry.path();

            let mut allowed = false;
            for allowed_file in &allowed_files {
                let filename = path.file_name().unwrap().to_str();
                if let Some(filename) = filename {
                    if filename.contains(allowed_file) {
                        allowed = true;
                        break;
                    }
                }
            }

            if !allowed
                && entry
                    .file_type()
                    .map(|f| f.is_file() || f.is_symlink())
                    .unwrap_or(false)
            {
                fs::remove_file(entry.path()).unwrap();
            }
        }
    }

    println!("cargo:rustc-link-search=native={}", lib_bin_dir.display());

    println!("cargo:rustc-link-lib=dylib=Ultralight");
    println!("cargo:rustc-link-lib=dylib=WebCore");
    println!("cargo:rustc-link-lib=dylib=AppCore");
}
