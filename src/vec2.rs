use core::ops;

pub const fn vec2<T>(x: T, y: T) -> Vec2<T> { Vec2 { x, y } }

#[derive(Copy,Clone,Default,Debug,PartialEq,Eq,Hash)]
#[repr(C)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Vec2<U> {
        vec2(f(self.x), f(self.y))
    }
    pub fn zip<U,V>(self, other: Vec2<U>, f: impl Fn(T,U) -> V) -> Vec2<V> {
        vec2(f(self.x, other.x), f(self.y, other.y))
    }
    pub fn reduce<U>(self, f: impl Fn(T,T) -> U) -> U {
        f(self.x, self.y)
    }
    pub fn as_slice(&self) -> &[T; 2] {
        unsafe { core::mem::transmute(self) }
    }
    pub fn as_slice_mut(&mut self) -> &mut [T; 2] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T> Vec2<T> where T: Clone, ops::Range<T>: Iterator<Item=T> {
    pub fn product_range(self, other: Vec2<T>) -> impl Iterator<Item=Self> {
        (self.x.clone()..other.x.clone()).flat_map(move |x| {
            (self.y.clone()..other.y.clone()).map(move |y| vec2(x.clone(),y))
        })
    }
}

impl<T> From<[T;2]> for Vec2<T> {
    fn from([x,y]: [T;2]) -> Self {
        Self { x, y }
    }
}

impl<T> ops::Index<usize> for Vec2<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T { &self.as_slice()[idx] }
}
impl<T> ops::IndexMut<usize> for Vec2<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T { &mut self.as_slice_mut()[idx] }
}

macro_rules! impl_ops {
    ($($norm:ident, $norm_fn:ident; $assign:ident, $assign_fn:ident; [ $op:tt $aop:tt ])+) => {
        $(
        impl<T: ops::$norm> ops::$norm for Vec2<T> {
            type Output = Vec2<T::Output>;
            fn $norm_fn(self, other: Self) -> Self::Output {
                vec2(self.x $op other.x, self.y $op other.y)
            }
        }
        impl<T: ops::$norm + Clone> ops::$norm<T> for Vec2<T> {
            type Output = Vec2<T::Output>;
            fn $norm_fn(self, other: T) -> Self::Output {
                vec2(self.x $op other.clone(), self.y $op other)
            }
        }
        impl<T: ops::$assign> ops::$assign for Vec2<T> {
            fn $assign_fn(&mut self, other: Self) {
                self.x $aop other.x;
                self.y $aop other.y;
            }
        }
        impl<T: ops::$assign + Clone> ops::$assign<T> for Vec2<T> {
            fn $assign_fn(&mut self, other: T) {
                self.x $aop other.clone();
                self.y $aop other;
            }
        }
        )+
    }
}

impl_ops! {
    Add, add; AddAssign, add_assign; [ + += ]
    Sub, sub; SubAssign, sub_assign; [ - -= ]

    Mul, mul; MulAssign, mul_assign; [ * *= ]
    Div, div; DivAssign, div_assign; [ / /= ]
    Rem, rem; RemAssign, rem_assign; [ % %= ]

    Shl, shl; ShlAssign, shl_assign; [ << <<= ]
    Shr, shr; ShrAssign, shr_assign; [ >> >>= ]

    BitAnd, bitand; BitAndAssign, bitand_assign; [ & &= ]
    BitOr,  bitor;  BitOrAssign,  bitor_assign;  [ | |= ]
    BitXor, bitxor; BitXorAssign, bitxor_assign; [ ^ ^= ]
}
