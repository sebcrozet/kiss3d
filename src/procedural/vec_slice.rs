//! Slicing on non-contiguous data.

pub struct VecSlice<'a, T> {
    data:   &'a [T],
    length: usize,
    stride: usize
}

pub struct VecSliceMut<'a, T> {
    data:   &'a mut [T],
    length: usize,
    stride: usize
}

impl<'a, T> Collection for VecSlice<'a, T> {
    fn len(&self) -> usize {
        self.length
    }

    fn is_empty(&self) -> bool {
        self.length != 0
    }
}

impl<'a, T> Collection for VecSliceMut<'a, T> {
    fn len(&self) -> usize {
        self.length
    }

    fn is_empty(&self) -> bool {
        self.length != 0
    }
}

impl<'a, T> VecSlice<'a, T> {
    #[inline]
    pub fn new(data: &'a [T], length: usize, stride: usize) -> VecSlice<'a, T> {
        assert!(stride > 0, "The stride must at least be 1.");
        assert!(length == 0 || data.len() >= 1 + (length - 1) * stride, "The data buffer is too small.");

        VecSlice {
            data:   data,
            length: length,
            stride: stride
        }
    }

    #[inline(always)]
    fn id(&self, i: usize) -> usize {
        i * self.stride
    }

    #[inline]
    pub fn get(&self, i: usize) -> &T {
        assert!(i < self.length);

        unsafe {
            self.get_unchecked(i)
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, i: usize) -> &T {
        self.data.get_unchecked(self.id(i))
    }
}

impl<'a, T> VecSliceMut<'a, T> {
    #[inline]
    pub fn new(data: &'a mut [T], length: usize, stride: usize) -> VecSliceMut<'a, T> {
        assert!(stride > 0, "The stride must at least be 1.");
        assert!(length == 0 || data.len() >= 1 + (length - 1) * stride, "The data buffer is too small.");

        VecSliceMut {
            data:   data,
            length: length,
            stride: stride
        }
    }

    #[inline(always)]
    fn id(&self, i: usize) -> usize {
        i * self.stride
    }

    #[inline]
    pub fn get(&self, i: usize) -> &T {
        assert!(i < self.length);

        unsafe {
            self.get_unchecked(i)
        }
    }

    #[inline]
    pub fn get_mut(&mut self, i: usize) -> &mut T {
        assert!(i < self.length);

        unsafe {
            self.get_unchecked_mut(i)
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, i: usize) -> &T {
        self.data.get_unchecked(self.id(i))
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, i: usize) -> &mut T {
        let id = self.id(i);
        self.data.unsafe_mut_ref(id)
    }
}

impl<'a, T: Clone> VecSliceMut<'a, T> {
    #[inline]
    pub fn copy_from(&mut self, data: &VecSlice<T>) {
        assert!(data.len() == self.len());

        for i in 0u .. data.len() {
            unsafe {
                *self.get_unchecked_mut(i) = data.get_unchecked(i).clone()
            }
        }
    }
}
