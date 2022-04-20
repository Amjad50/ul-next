macro_rules! failwith {
    (
        $expr:expr $(, $($extra:tt)* )?
    ) => ({
        eprintln!(
            "c_callback error: {}",
            format_args!($expr $(, $($extra)* )?),
        );
        std::process::exit(1);
    })
}

macro_rules! ffi_unwrap {
    ($expr:expr, $msg:expr $(,)?) => {
        if let Some(inner) = $expr {
            inner
        } else {
            failwith!($msg)
        }
    };
}

macro_rules! c_callback {
    {
        $(#[$attr:meta])*
        $vis:vis unsafe extern "C" fn $name:ident ($($c_arg:ident: $c_arg_ty:ty),*) $(: ($($arg:ident: $arg_ty:ty),*))?;
    } => {
        // Source: https://users.rust-lang.org/t/callback-based-c-ffi/26583/5
        //
        // This could be seen as a "function constructor":
        // for each concrete Env type parameter,
        // a new static function is defined by monomorphisation
        $(#[$attr])*
        $vis unsafe extern "C" fn $name<Env: ::std::marker::Sized>(callback_data: *mut ::std::ffi::c_void, $($c_arg: $c_arg_ty),*)
            where
                Env: FnMut($($($arg_ty),*)?),
        {
            // Prevent unwinding accross the FFI
            ::scopeguard::defer_on_unwind!({
                ::std::process::abort();
            });

            let at_env_raw_ptr: *mut ::std::boxed::Box<Env> = callback_data as *mut ::std::boxed::Box<Env>;
            let callback: &mut ::std::boxed::Box<Env> = ffi_unwrap!(at_env_raw_ptr.as_mut(), "null ptr",);

            // For each given Env type parameter,
            // Rust knows how to call this since it is using the static address
            // <Env as FnMut<_>>::call_mut(at_env, result, data)
            // (this is the only part of the code that depends on the Env type)
            callback($($($arg),*)?);
        }
    };
}
