use std::iter::FromIterator;
use std::iter::Sum;
use std::ops::{Add, AddAssign};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rating {
    mean: f64,
    variance: f64,
}

impl Eq for Rating {}

impl Rating {
    #[inline]
    pub const fn new(mean: f64, variance: f64) -> Self {
        Self { mean, variance }
    }

    #[inline]
    pub const fn mean(&self) -> f64 {
        self.mean
    }

    #[inline]
    pub const fn variance(&self) -> f64 {
        self.variance
    }
}

macro_rules! impl_add {
    (
        $(($l:ty,$r:ty))*
    ) => {
        $(
            impl Add<$r> for $l {
                type Output = Rating;

                fn add(self, other: $r) -> Self::Output {
                    Self::Output {
                        mean: self.mean + other.mean,
                        variance: self.variance + other.variance,
                    }
                }
            }
        )*
    };
}

impl_add! {
    (Rating, Rating)
    (Rating, &Rating)
    (&Rating, Rating)
    (&Rating, &Rating)
}

impl AddAssign<Self> for Rating {
    fn add_assign(&mut self, other: Self) {
        self.mean += other.mean;
        self.variance += other.variance;
    }
}

impl Sum<Self> for Rating {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(0.0, 0.0), |a, b| a + b)
    }
}

impl<'a> Sum<&'a Self> for Rating {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::new(0.0, 0.0), |a, &b| a + b)
    }
}

impl FromIterator<Self> for Rating {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        iter.into_iter().sum()
    }
}

impl<'a> FromIterator<&'a Self> for Rating {
    fn from_iter<I: IntoIterator<Item = &'a Self>>(iter: I) -> Self {
        iter.into_iter().sum()
    }
}
