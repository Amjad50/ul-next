fn main() {
    // skip if we are building doc
    #[cfg(all(not(docsrs), any(feature = "linked", feature = "appcore_linked")))]
    {
        use std::path::PathBuf;
        use std::{env, fs};

        const REV: &str = "158d65c";

        fn platform() -> &'static str {
            let target_os = env::var("CARGO_CFG_TARGET_OS").expect("TARGET_OS not set");
            let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("TARGET_ARCH not set");

            match (target_os.as_str(), target_arch.as_str()) {
                ("windows", "x86_64") => "win-x64",
                ("windows", _) => panic!("Only x86_64 is supported on Windows"),
                ("linux", "x86_64") => "linux-x64",
                ("linux", "aarch64") => "linux-arm64",
                ("linux", _) => panic!("Only x86_64 and aarch64 are supported on Linux"),
                ("macos", "x86_64") => "mac-x64",
                ("macos", "aarch64") => "mac-arm64",
                ("macos", _) => panic!("Only x86_64 and aarch64 are supported on MacOS"),
                (_, _) => panic!("Only Windows, Linux and MacOS are supported"),
            }
        }

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let sdk_dir = out_dir.join("ul-sdk");

        println!("cargo:rerun-if-changed=build.rs");

        if sdk_dir.is_dir() {
            fs::remove_dir_all(&sdk_dir).expect("Could not remove already existing ultralight sdk");
        }
        fs::create_dir_all(&sdk_dir).expect("Could not create ultralight sdk directory");

        let sdk_url = format!(
            "https://ultralight-sdk-dev.sfo2.cdn.digitaloceanspaces.com/ultralight-sdk-{REV}-{}.7z",
            platform()
        );

        eprintln!("Downloading Ultralight SDK from {}", sdk_url);
        eprintln!("sdk_dir: {:?}", sdk_dir);

        // use ureq
        let response = ureq::get(&sdk_url).call();
        match response {
            Ok(response) => {
                let status = response.status();
                if status == 200 {
                    let mut tmp_file = fs::File::create(sdk_dir.join("ul-sdk.7z")).unwrap();
                    std::io::copy(&mut response.into_reader(), &mut tmp_file).unwrap();
                    sevenz_rust::decompress_file(sdk_dir.join("ul-sdk.7z"), &sdk_dir).unwrap();
                } else {
                    panic!("Could not download Ultralight SDK, status code: {}", status);
                }
            }
            Err(err) => {
                panic!("Could not download Ultralight SDK: {}", err);
            }
        }

        let bin_dir = sdk_dir.join("bin");
        let lib_dir = sdk_dir.join("lib");

        if cfg!(feature = "only-ul-deps") {
            let allowed_files = ["Ultralight", "UltralightCore", "WebCore", "AppCore"];
            for entry in fs::read_dir(&bin_dir).unwrap().flatten() {
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

        println!("cargo:rustc-link-search=native={}", bin_dir.display());
        // // for windows only
        println!("cargo:rustc-link-search=native={}", lib_dir.display());

        println!("cargo:rustc-link-lib=dylib=Ultralight");
        println!("cargo:rustc-link-lib=dylib=WebCore");
        #[cfg(feature = "appcore_linked")]
        {
            println!("cargo:rustc-link-lib=dylib=AppCore");
        }
    }
}
