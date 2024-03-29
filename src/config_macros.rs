macro_rules! set_config (
    ($config: expr, $config_item: expr, $ffiName:ident) => (
        if let Some(config_item) = $config_item {
            unsafe {
                ::ul_sys::$ffiName($config, config_item);
            }
        }
    )
);

macro_rules! set_config_str (
    ($config: expr, $config_item: expr, $ffiName:ident) => (
        if let Some(config_item) = $config_item {
            unsafe {
                let cstr = ::std::ffi::CString::new(
                    config_item
                ).unwrap();
                let str = ::ul_sys::ulCreateString(
                    cstr.as_ptr()
                );

                ::ul_sys::$ffiName($config, str);
            }
        }
    )
);
