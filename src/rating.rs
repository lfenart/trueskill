use num_traits::Float;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rating<F>
where
    F: Float,
{
    mu: F,
    sigma: F,
}

impl<F: Float> Rating<F> {
    #[inline]
    pub fn new(mu: F, sigma: F) -> Self {
        Self { mu, sigma }
    }

    #[inline]
    pub fn mu(&self) -> F {
        self.mu
    }

    #[inline]
    pub fn sigma(&self) -> F {
        self.sigma
    }
}

impl<T, F: Float> From<T> for Rating<F>
where
    T: std::convert::AsRef<[Rating<F>]>,
{
    #[inline]
    fn from(team: T) -> Self {
        let (mu, sigma2) =
            team.as_ref()
                .iter()
                .fold((F::zero(), F::zero()), |(mu, sigma2), x: &Self| {
                    let sigma = x.sigma();
                    (mu + x.mu(), sigma2 + sigma * sigma)
                });
        Self::new(mu, sigma2.sqrt())
    }
}
