use core::iter::FromIterator;
use core::iter::Sum;
use core::ops::{Add, AddAssign};

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

#[cfg(test)]
mod test {
    use super::*;

    static MEAN: f64 = 3.0;
    static VARIANCE: f64 = 0.5;
    static RATING: Rating = Rating::new(MEAN, VARIANCE);
    static RATING2: Rating = Rating::new(2.0, 1.0);

    #[test]
    fn mean() {
        assert_eq!(RATING.mean(), MEAN);
    }

    #[test]
    fn variance() {
        assert_eq!(RATING.variance(), VARIANCE);
    }

    #[test]
    fn add() {
        let rating = RATING + RATING2;
        assert_eq!(rating.mean(), MEAN + RATING2.mean());
        assert_eq!(rating.variance(), VARIANCE + RATING2.variance());
    }

    #[test]
    fn from_iter() {
        let ratings = [RATING, RATING2];
        let rating = Rating::from_iter(ratings.iter());
        assert_eq!(rating.mean(), MEAN + RATING2.mean());
        assert_eq!(rating.variance(), VARIANCE + RATING2.variance());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        let text = serde_json::to_string(&RATING).unwrap();
        assert_eq!(text, r#"{"mean":3.0,"variance":0.5}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize() {
        let rating = serde_json::from_str::<Rating>(r#"{"mean":3.0,"variance":0.5}"#).unwrap();
        assert_eq!(rating, RATING);
    }
}
