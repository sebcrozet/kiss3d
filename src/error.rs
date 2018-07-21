#![macro_use]

#[allow(unused_macros)]
macro_rules! verify(
    ($e: expr) => {
        {
            let res = $e;
            #[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
            { assert_eq!(::context::Context::get().get_error(), 0); }
            res
        }
    }
);

#[allow(unused_macros)]
macro_rules! ignore(
    ($e: expr) => {
        {
            let res = $e;
            #[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
            { let _ = ::context::Context::get().get_error(); }
            res
        }
    }
);

#[allow(unused_macros)]
macro_rules! checked(
    ($e: expr) => {
        {
            let res = $e;
            if cfg!(not(any(target_arch = "wasm32", target_arch = "asmjs"))) && ::context::Context::get().get_error() != 0 {
                None
            } else {
                Some(res)
            }
        }
    }
);
