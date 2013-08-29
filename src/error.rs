#[macro_escape];

macro_rules! verify(
    ($e: expr) => (
        {
            $e;
            assert_eq!(gl::GetError(), 0);
        }
    )
)
