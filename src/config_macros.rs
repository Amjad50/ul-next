macro_rules! set_config (
    ($config: expr, $config_item: expr, $lib: ident . $($ffiName:tt)+) => (
        if let Some(config_item) = $config_item {
            unsafe {
                $lib.$($ffiName)+($config, config_item);
            }
        }
    )
);

macro_rules! set_config_str (
    ($config: expr, $config_item: expr, $lib: ident . $($ffiName:tt)+) => (
        if let Some(config_item) = $config_item {
            unsafe {
                let cstr = ::std::ffi::CString::new(
                    config_item
                ).unwrap();
                let str = $lib.ultralight().ulCreateString(
                    cstr.as_ptr()
                );

                $lib.$($ffiName)+($config, str);
            }
        }
    )
);
