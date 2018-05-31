#![macro_use]

macro_rules! verify(
    ($e: expr) => {
        unsafe {
            let res = $e;
            assert_eq!(::context::Context::get().get_error(), 0);
            res
        }
    }
);
