use num_traits::Float;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rating<F>
where
    F: Float,
{
    mu: F,
    sigma: F,
}

impl<F: Float> Rating<F> {
    pub fn new(mu: F, sigma: F) -> Self {
        Self { mu, sigma }
    }

    pub fn mu(&self) -> F {
        self.mu
    }

    pub fn sigma(&self) -> F {
        self.sigma
    }
}

impl<T, F: Float> From<T> for Rating<F>
where
    T: std::convert::AsRef<[Rating<F>]>,
{
    fn from(team: T) -> Rating<F> {
        let (mu, sigma2) =
            team.as_ref()
                .iter()
                .fold((F::zero(), F::zero()), |(mu, sigma2), x: &Rating<F>| {
                    (mu + x.mu(), sigma2 + x.sigma().powf(F::from(2.).unwrap()))
                });
        Rating::new(F::from(mu).unwrap(), F::from(sigma2.sqrt()).unwrap())
    }
}
