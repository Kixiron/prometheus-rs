use std::{
    fmt::{self, Write},
    ops,
    sync::atomic::{AtomicI64, AtomicU64, Ordering},
};

#[derive(Debug)]
#[repr(transparent)]
pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    #[inline]
    pub const fn zeroed() -> Self {
        Self(AtomicU64::new(0))
    }

    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self(AtomicU64::new(bits))
    }

    #[inline]
    pub fn fetch_add(&self, val: f64, order: Ordering) -> f64 {
        loop {
            let current = self.0.load(order);
            let new = f64::from_bits(current) + val;

            if self.0.compare_and_swap(current, f64::to_bits(new), order) == current {
                break new;
            }
        }
    }

    #[inline]
    pub fn fetch_sub(&self, val: f64, order: Ordering) -> f64 {
        loop {
            let current = self.0.load(order);
            let new = f64::from_bits(current) - val;

            if self.0.compare_and_swap(current, f64::to_bits(new), order) == current {
                break new;
            }
        }
    }

    #[inline]
    pub fn store(&self, val: f64, order: Ordering) {
        loop {
            let current = self.0.load(order);

            if self.0.compare_and_swap(current, f64::to_bits(val), order) == current {
                break;
            }
        }
    }

    #[inline]
    pub fn load(&self, order: Ordering) -> f64 {
        f64::from_bits(self.0.load(order))
    }
}

pub trait Num:
    Copy + ops::Add + ops::AddAssign + ops::Sub + Default + PartialEq + PartialOrd + fmt::Debug
{
    fn from_u64(int: u64) -> Self;
}

pub trait AtomicNum {
    type Type: Num;

    fn new() -> Self;
    fn inc(&self);
    fn inc_by(&self, inc: Self::Type);
    fn dec(&self);
    fn dec_by(&self, dec: Self::Type);
    fn set(&self, val: Self::Type);
    fn get(&self) -> Self::Type;
    fn clear(&self);
    fn format(int: Self::Type, f: &mut String, quotes: bool) -> fmt::Result;
}

macro_rules! impl_atomic {
    ($($atomic:ty := $new:expr => $ty:ty = $fmt:expr,)*) => {
        $(
            impl Num for $ty {
                #[inline(always)]
                fn from_u64(int: u64) -> Self {
                    int as $ty
                }
            }

            impl AtomicNum for $atomic {
                type Type = $ty;

                /// Create a new `AtomicNum`
                fn new() -> Self {
                    $new
                }

                /// Increment the value by 1
                fn inc(&self) {
                    self.fetch_add(1 as _, Ordering::SeqCst);
                }

                /// Increment the value by `inc`
                fn inc_by(&self, inc: Self::Type) {
                    self.fetch_add(inc, Ordering::SeqCst);
                }

                /// Decrement the value by 1
                fn dec(&self) {
                    self.fetch_sub(1 as _, Ordering::SeqCst);
                }

                /// Decrement the value by `dec`
                fn dec_by(&self, dec: Self::Type) {
                    self.fetch_sub(dec, Ordering::SeqCst);
                }

                /// Set the value to `val`
                fn set(&self, val: Self::Type) {
                    self.store(val, Ordering::SeqCst);
                }

                /// Get the current value
                fn get(&self) -> Self::Type {
                    self.load(Ordering::SeqCst)
                }

                /// Reset the value to 0
                fn clear(&self) {
                    self.store(0 as _, Ordering::SeqCst);
                }

                fn format(int: Self::Type, f: &mut String, quotes: bool) -> fmt::Result {
                    let fmt: fn(&mut String, Self::Type, bool) -> fmt::Result = $fmt;
                    fmt(f, int, quotes)
                }
            }
        )*
    };
}

// Implement `AtomicNum` and `Num` for all data types
impl_atomic! {
    AtomicU64 := AtomicU64::new(0) => u64 = |f, int, quotes| {
        if quotes {
            write!(f, "\"{:?}\"", int)
        } else {
            write!(f, "{:?}", int)
        }
    },
    AtomicI64 := AtomicI64::new(0) => i64 = |f, int, quotes| {
        if quotes {
            write!(f, "\"{:?}\"", int)
        } else {
            write!(f, "{:?}", int)
        }
    },
    AtomicF64 := AtomicF64::zeroed() => f64 = |f, int, quotes| {
        if quotes {
            match int {
                int if int.is_infinite() && int.is_sign_positive() => write!(f, "\"+Inf\""),
                int if int.is_infinite() && int.is_sign_negative() => write!(f, "\"-Inf\""),
                int if int.is_nan()  => write!(f, "\"Nan\""),
                int => write!(f, "\"{:?}\"", int),
            }
        } else {
            match int {
                int if int.is_infinite() && int.is_sign_positive() => write!(f, "+Inf"),
                int if int.is_infinite() && int.is_sign_negative() => write!(f, "-Inf"),
                int if int.is_nan()  => write!(f, "Nan"),
                int => write!(f, "{:?}", int),
            }
        }
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroed_is_zero() {
        static ZERO: AtomicF64 = AtomicF64::zeroed();

        assert_eq!(0.0, ZERO.load(Ordering::SeqCst));
    }

    #[test]
    fn adding() {
        static FLOAT: AtomicF64 = AtomicF64::zeroed();

        assert_eq!(FLOAT.fetch_add(1.0, Ordering::SeqCst), 1.0);
        assert_eq!(1.0, FLOAT.load(Ordering::SeqCst));

        assert_eq!(FLOAT.fetch_add(10.0, Ordering::SeqCst), 11.0);
        assert_eq!(11.0, FLOAT.load(Ordering::SeqCst));

        assert_eq!(FLOAT.fetch_add(0.0, Ordering::SeqCst), 11.0);
        assert_eq!(11.0, FLOAT.load(Ordering::SeqCst));
    }

    #[test]
    fn subtracting() {
        static ZERO: AtomicF64 = AtomicF64::zeroed();

        assert_eq!(ZERO.fetch_sub(1.0, Ordering::SeqCst), -1.0);
        assert_eq!(-1.0, ZERO.load(Ordering::SeqCst));

        assert_eq!(ZERO.fetch_sub(10.0, Ordering::SeqCst), -11.0);
        assert_eq!(-11.0, ZERO.load(Ordering::SeqCst));

        assert_eq!(ZERO.fetch_sub(0.0, Ordering::SeqCst), -11.0);
        assert_eq!(-11.0, ZERO.load(Ordering::SeqCst));

        static ONE: AtomicF64 = AtomicF64::from_bits(0x3FF0000000000000);
        assert_eq!(ONE.load(Ordering::SeqCst), 1.0);

        assert_eq!(ONE.fetch_sub(1.0, Ordering::SeqCst), 0.0);
        assert_eq!(0.0, ONE.load(Ordering::SeqCst));

        assert_eq!(ONE.fetch_sub(0.0, Ordering::SeqCst), 0.0);
        assert_eq!(0.0, ONE.load(Ordering::SeqCst));
    }

    #[test]
    fn storing() {
        static FLOAT: AtomicF64 = AtomicF64::zeroed();

        FLOAT.store(1000.034512, Ordering::SeqCst);
        assert_eq!(FLOAT.load(Ordering::SeqCst), 1000.034512);

        FLOAT.store(-1000.034512, Ordering::SeqCst);
        assert_eq!(FLOAT.load(Ordering::SeqCst), -1000.034512);
    }
}
