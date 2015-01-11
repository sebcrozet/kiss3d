#![macro_use]

macro_rules! verify(
    ($e: expr) => {
        unsafe {
            let res = $e;
            assert_eq!(gl::GetError(), 0);
            res
        }
    }
);
