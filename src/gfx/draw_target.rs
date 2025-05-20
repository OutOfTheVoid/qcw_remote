pub trait Mask {
    fn get_mask(&self, position: (isize, isize)) -> bool;
}

pub trait DrawTarget {
    fn set_pixel(&mut self, position: (isize, isize), color: bool);
    fn get_pixel(&self, position: (isize, isize)) -> bool;
}

pub trait _DTRef: DrawTarget + Sized {
    fn dt_ref<'a> (&'a mut self) -> DTRef<'a, Self>;
}

impl<T: DrawTarget + Sized> _DTRef for T {
    fn dt_ref<'a> (&'a mut self) -> DTRef<'a, Self> {
        DTRef { dt: self }
    }
}

pub struct DTRef<'a, T: DrawTarget> {
    dt: &'a mut T
}

impl<T: DrawTarget> DrawTarget for DTRef<'_, T> {
    fn get_pixel(&self, position: (isize, isize)) -> bool {
        self.dt.get_pixel(position)
    }

    fn set_pixel(&mut self, position: (isize, isize), color: bool) {
        self.dt.set_pixel(position, color);
    }
}

pub struct TranslatedDrawTarget<Inner: DrawTarget> {
    inner: Inner,
    offset: (isize, isize)
}

impl <Inner: DrawTarget> TranslatedDrawTarget<Inner> {
    pub fn new(inner: Inner, offset: (isize, isize)) -> Self {
        Self {
            inner,
            offset
        }
    }
}

impl<'a, Inner: DrawTarget> DrawTarget for TranslatedDrawTarget<Inner> {
    fn set_pixel(&mut self, position: (isize, isize), color: bool) {
        self.inner.set_pixel((position.0 + self.offset.0, position.1 + self.offset.1), color);
    }

    fn get_pixel(&self, position: (isize, isize)) -> bool {
        self.inner.get_pixel((position.0 + self.offset.0, position.1 + self.offset.1))
    }
}

pub trait _Translatable: DrawTarget + Sized {
    fn translate(self, offset: (isize, isize)) -> TranslatedDrawTarget<Self>;
}

impl<T: Sized + DrawTarget> _Translatable for T {
    fn translate(self, offset: (isize, isize)) -> TranslatedDrawTarget<Self> {
        TranslatedDrawTarget { inner: self, offset }
    }
}


pub struct MaskedDrawTarget<Inner: DrawTarget, M: Mask> {
    inner: Inner,
    mask: M,
}

impl<Inner: DrawTarget, M: Mask> MaskedDrawTarget<Inner, M> {
    pub fn new(inner: Inner, mask: M) -> Self {
        Self {
            inner,
            mask,
        }
    }
}

impl<Inner: DrawTarget, M: Mask> DrawTarget for MaskedDrawTarget<Inner, M> {
    fn set_pixel(&mut self, position: (isize, isize), color: bool) {
        if self.mask.get_mask(position) {
            self.inner.set_pixel(position, color);
        }
    }

    fn get_pixel(&self, position: (isize, isize)) -> bool {
        self.inner.get_pixel(position)
    }
}

pub struct RectMask {
    pub upper_left: (isize, isize),
    pub lower_right: (isize, isize)
}

impl Mask for RectMask {
    fn get_mask(&self, position: (isize, isize)) -> bool {
        (self.upper_left.0..=self.lower_right.0).contains(&position.0) &&
        (self.upper_left.1..=self.lower_right.1).contains(&position.1)
    }
}

pub trait _Maskable: DrawTarget + Sized {
    fn mask<'a, M: Mask>(self, mask: M) -> MaskedDrawTarget<Self, M>;
}

impl<T: Sized + DrawTarget> _Maskable for T {
    fn mask<'a, M: Mask>(self, mask: M) -> MaskedDrawTarget<Self, M> {
        MaskedDrawTarget { inner: self, mask }
    }
}
