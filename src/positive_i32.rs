use std::convert::TryFrom;
use std::num::{NonZeroI32, NonZeroU32};
use std::ops::Neg;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PositiveI32(NonZeroI32);

impl PositiveI32 {
    pub const fn from_i32(value: i32) -> Option<Self> {
        if value > 0 {
            // SAFETY: `value` is nonzero and positive.
            let value = unsafe { NonZeroI32::new_unchecked(value) };
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn from_u32(value: u32) -> Option<Self> {
        if value > 0 && value <= i32::MAX as u32 {
            // SAFETY: `value as i32` is nonzero and positive.
            let value = unsafe { NonZeroI32::new_unchecked(value as i32) };
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn as_non_zero_i32(self) -> NonZeroI32 {
        self.0
    }

    pub const fn as_i32(self) -> i32 {
        self.0.get()
    }

    pub const fn as_non_zero_u32(self) -> NonZeroU32 {
        let as_u32 = self.0.get() as u32;
        // SAFETY: `as_u32` is nonzero.
        unsafe { NonZeroU32::new_unchecked(as_u32) }
    }

    pub const fn as_u32(self) -> u32 {
        self.0.get() as u32
    }

    pub const fn negated(self) -> NonZeroI32 {
        let result = -self.0.get();
        // SAFETY: `result` is nonzero.
        unsafe { NonZeroI32::new_unchecked(result) }
    }
}

impl TryFrom<i32> for PositiveI32 {
    type Error = ();

    fn try_from(value: i32) -> Result<PositiveI32, ()> {
        Self::from_i32(value).ok_or(())
    }
}

impl TryFrom<u32> for PositiveI32 {
    type Error = ();

    fn try_from(value: u32) -> Result<PositiveI32, ()> {
        Self::from_u32(value).ok_or(())
    }
}

impl Into<i32> for PositiveI32 {
    fn into(self) -> i32 {
        self.as_i32()
    }
}

impl Into<u32> for PositiveI32 {
    fn into(self) -> u32 {
        self.as_u32()
    }
}

impl Into<NonZeroI32> for PositiveI32 {
    fn into(self) -> NonZeroI32 {
        self.as_non_zero_i32()
    }
}

impl Into<NonZeroU32> for PositiveI32 {
    fn into(self) -> NonZeroU32 {
        self.as_non_zero_u32()
    }
}

impl Neg for PositiveI32 {
    type Output = NonZeroI32;

    fn neg(self) -> NonZeroI32 {
        NonZeroI32::new(-self.0.get()).unwrap()
    }
}
