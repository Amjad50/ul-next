macro_rules! failwith {
    (
        $expr:expr $(, $($extra:tt)* )?
    ) => ({
        eprintln!(
            "c_callback error: {}",
            format_args!($expr $(, $($extra)* )?),
        );
        ::std::process::exit(1);
    })
}

macro_rules! ffi_unwrap {
    ($expr:expr, $msg:expr $(,)?) => {
        if let ::std::option::Option::Some(inner) = $expr {
            inner
        } else {
            failwith!($msg)
        }
    };
}

macro_rules! c_callback {
    {
        $(#[$attr:meta])*
        $vis:vis unsafe extern "C" fn $name:ident ($($c_arg:ident: $c_arg_ty:ty),*) $(-> $c_ret_ty:ty)? $(: ($($arg:ident: $arg_ty:ty),*))? $(=> $ret:ident: $ret_ty:ty)?
        {
            $($body:tt)*
        }
        $($ret_body:block)?
    } => {
        // Source: https://users.rust-lang.org/t/callback-based-c-ffi/26583/5
        //
        // This could be seen as a "function constructor":
        // for each concrete Env type parameter,
        // a new static function is defined by monomorphisation
        $(#[$attr])*
        $vis unsafe extern "C" fn $name<Env>(callback_data: *mut ::std::ffi::c_void, $($c_arg: $c_arg_ty),*)
            $(-> $c_ret_ty)?
            where
                Env: ::std::ops::FnMut($($($arg_ty),*)?) $(-> $ret_ty)? + 'static,
        {
            // Prevent unwinding accross the FFI
            ::scopeguard::defer_on_unwind!({
                ::std::process::abort();
            });

            let at_env_raw_ptr: *mut ::std::boxed::Box<Env> = callback_data as *mut ::std::boxed::Box<Env>;
            let callback: &mut ::std::boxed::Box<Env> = ffi_unwrap!(at_env_raw_ptr.as_mut(), "null ptr",);

            $($body)*

            // For each given Env type parameter,
            // Rust knows how to call this since it is using the static address
            // <Env as FnMut<_>>::call_mut(at_env, result, data)
            // (this is the only part of the code that depends on the Env type)
            let _ret = callback($($($arg),*)?);

            $(let $ret = _ret;)?
            $(let _ret = $ret_body;)?
            _ret
        }
    };
}

macro_rules! set_callback {
    {
        $(#[$attr:meta])*
        $vis:vis fn $name:ident(&self, callback: FnMut($($($arg:ident: $arg_ty:ty),+)?) $(-> $ret:ident: $ret_ty:ty)?):
              $ul_callback_setter:ident($($ul_arg:ident: $ul_arg_ty:ty),*) $(-> $ul_ret_ty:ty)?
        {
            $($body:tt)*
        }
        $($ret_body:block)?
    } => {
        $(#[$attr])*
        $vis fn $name<F>(&self, callback: F)
        where
            F: ::std::ops::FnMut($($($arg_ty),*)?) $(-> $ret_ty)? + 'static,
        {
            c_callback! {
                unsafe extern "C" fn trampoline($($ul_arg: $ul_arg_ty),*) $(-> $ul_ret_ty)? $(: ($($arg: $arg_ty),+))? $(=> $ret: $ret_ty)?
                {
                    $($body)*
                }
                $($ret_body)?
            }

            // Note that we need to double-box the callback, because a `*mut FnMut()` is a fat pointer
            // that can't be cast to a `*const c_void`.
            //
            // Note that we leak the box here, which will result in memory leak
            // as we can't get hold of the data later and free it. But since
            // setting the handler is only done once, this may not be a big problem.
            // FIXME: should we store it in instead?
            let callback: *mut ::std::boxed::Box<F> =
                ::std::boxed::Box::into_raw(::std::boxed::Box::new(::std::boxed::Box::new(callback)));
            let data = callback as *mut ::std::ffi::c_void;

            // SAFETY: We're passing a pointer to a function that has a static lifetime.
            unsafe {
                ul_sys::$ul_callback_setter(
                    self.internal,
                    Some(trampoline::<F>),
                    data
                );
            }
        }
    };
}
