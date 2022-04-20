use std::{ffi::c_void, panic, process};

// Source: https://users.rust-lang.org/t/callback-based-c-ffi/26583/5
//
// This could be seen as a "function constructor":
// for each concrete Env type parameter,
// a new static function is defined by monomorphisation
pub unsafe extern "C" fn c_callback<Env: Sized>(callback_data: *mut c_void)
where
    Env: FnMut() + 'static + Send + panic::RefUnwindSafe,
{
    // Prevent unwinding accross the FFI
    ::scopeguard::defer_on_unwind!({
        process::abort();
    });

    macro_rules! failwith {
            (
                $expr:expr $(, $($extra:tt)* )?
            ) => ({
                eprintln!(
                    "c_callback error: {}",
                    format_args!($expr $(, $($extra)* )?),
                );
                dbg!((
                    callback_data,
                ));
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

    let at_env_raw_ptr: *mut Box<Env> = callback_data as *mut Box<Env>;
    let at_env: &mut Box<Env> = ffi_unwrap!(at_env_raw_ptr.as_mut(), "null ptr",);

    // For each given Env type parameter,
    // Rust knows how to call this since it is using the static address
    // <Env as FnMut<_>>::call_mut(at_env, result, data)
    // (this is the only part of the code that depends on the Env type)
    at_env();
}
