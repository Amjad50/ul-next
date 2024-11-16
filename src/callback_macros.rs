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
        $vis:vis unsafe extern "C" fn $name:ident $([$myself:ident])? ($($c_arg:ident: $c_arg_ty:ty),*) $(-> $c_ret_ty:ty)? $(: ($($arg:ident: $arg_ty:ty),*))? $(=> $ret:ident: $ret_ty:ty)?
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

            let at_env_raw_ptr: *mut ::std::boxed::Box<Holder<Env>> = callback_data as *mut ::std::boxed::Box<Holder<Env>>;
            let holder: &mut ::std::boxed::Box<Holder<Env>> = ffi_unwrap!(at_env_raw_ptr.as_mut(), "null ptr",);
            let callback = &mut holder.callback;
            $(let $myself = &holder.myself;)?

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
              [$selfty:tt :: $($lib:tt)+]$([$myself:ident])? $ul_callback_setter:ident($($ul_arg:ident: $ul_arg_ty:ty),*) $(-> $ul_ret_ty:ty)?
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
            // FIXME: this is a bit effy, not sure if its safe to include a local lifetime here
            //        its not producing error because we are casting it to raw ptr and back
            //        so it loses track of the lifetime
            //        check this later and make sure its safe. or change it.
            struct Holder<'a, T> {
                #[allow(dead_code)]
                myself: &'a $selfty,
                callback: T,
            }

            c_callback! {
                unsafe extern "C" fn trampoline$([$myself])?($($ul_arg: $ul_arg_ty),*) $(-> $ul_ret_ty)? $(: ($($arg: $arg_ty),+))? $(=> $ret: $ret_ty)?
                {
                    $($body)*
                }
                $($ret_body)?
            }

            let holder_data = Holder {
                myself: self,
                callback,
            };

            // Note that we need to double-box the callback, because a `*mut FnMut()` is a fat pointer
            // that can't be cast to a `*const c_void`.
            //
            // Note that we leak the box here, which will result in memory leak
            // as we can't get hold of the data later and free it. But since
            // setting the handler is only done once, this may not be a big problem.
            // FIXME: should we store it in instead?
            let holder: *mut ::std::boxed::Box<Holder<F>> =
                ::std::boxed::Box::into_raw(::std::boxed::Box::new(::std::boxed::Box::new(holder_data)));
            let data = holder as *mut ::std::ffi::c_void;

            // SAFETY: We're passing a pointer to a function that has a static lifetime.
            unsafe {
                self.$($lib)+.$ul_callback_setter(
                    self.internal,
                    Some(trampoline::<F>),
                    data
                );
            }
        }
    };
}

// note that `arg_ty` and `ret_ty` are not used in the macro, but are there
// just for clarification and to make implementations of the macro easy
macro_rules! platform_set_interface_macro {
    {
        $(#[$attr:meta])*
        $vis:vis $setter_name:ident<$rust_ty:ident>($lib:ident, $setter_arg_name:ident -> $static_name:ident) -> $ul_setter:ident($ul_struct_arg_ty:ident)
        {
            $(
                $fn_name:ident($( ( $($ul_arg:ident: $ul_arg_ty:ty),* ) $(-> $ul_ret_ty:ty)? )?) ->  ($(($($arg:ident: $arg_ty:ty),*) $(-> $ret:ident: $ret_ty:ty)? )? )
                {
                    $($from_ul_to_rs_body:tt)*
                }
                $($from_rs_to_ul_body:block)?
            )+
        }
    } => {
        $(#[$attr])*
        $vis fn $setter_name<T: $rust_ty + Send + 'static>(lib: ::std::sync::Arc<$crate::Library>, $setter_arg_name: T) {
            let $setter_arg_name = Box::new($setter_arg_name);
            $static_name.lib.lock().unwrap().replace(lib.clone());
            $static_name.obj.lock().unwrap().replace($setter_arg_name);

            $(
                unsafe extern "C" fn $fn_name($($($ul_arg: $ul_arg_ty),*)?) $($(-> $ul_ret_ty)?)? {
                    #[allow(unused_variables)]
                    let $lib = $static_name.lib.lock().unwrap().as_ref().unwrap().clone();
                    $($from_ul_to_rs_body)*

                    let mut $setter_arg_name = $static_name.obj.lock().unwrap();
                    // the $setter_arg_name must always be `Some` at this point.
                    let _r $($(: $ret_ty)?)? = $setter_arg_name.as_mut().unwrap().$fn_name($($($arg),*)?);
                    $($(let $ret = _r;)?)?
                    $(
                    let _r = $from_rs_to_ul_body;
                    )?
                    _r
                }
            )+

            let ul_struct = ul_sys::$ul_struct_arg_ty {
                $(
                    $fn_name: Some($fn_name)
                ),+
            };

            unsafe {
                lib.ultralight().$ul_setter(ul_struct);
            }
        }
    };
}
