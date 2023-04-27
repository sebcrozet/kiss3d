#[macro_export]
macro_rules! verify(
    ($e: expr) => {
        {
            let res = $e;
            #[cfg(not(target_arch = "wasm32"))]
            { assert_eq!($crate::context::Context::get().get_error(), 0); }
            res
        }
    }
);

#[macro_export]
macro_rules! ignore(
    ($e: expr) => {
        {
            let res = $e;
            #[cfg(not(target_arch = "wasm32"))]
            { let _ = $crate::context::Context::get().get_error(); }
            res
        }
    }
);

#[macro_export]
macro_rules! checked(
    ($e: expr) => {
        {
            let res = $e;
            if cfg!(not(any(target_arch = "wasm32", target_arch = "asmjs"))) && $crate::context::Context::get().get_error() != 0 {
                None
            } else {
                Some(res)
            }
        }
    }
);
