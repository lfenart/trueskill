#[derive(Clone, Copy, Debug)]
pub struct Rating {
    mu: f32,
    sigma: f32,
}

impl Rating {
    pub const fn new(mu: f32, sigma: f32) -> Self {
        Self { mu, sigma }
    }

    pub const fn mu(&self) -> f32 {
        self.mu
    }

    pub const fn sigma(&self) -> f32 {
        self.sigma
    }
}

impl<T> From<T> for Rating
where
    T: std::convert::AsRef<[Rating]>,
{
    fn from(team: T) -> Rating {
        let (mu, sigma2) = team
            .as_ref()
            .iter()
            .fold((0., 0.), |(mu, sigma2), x: &Rating| {
                (mu + x.mu(), sigma2 + x.sigma().powf(2f32))
            });
        Rating::new(mu, sigma2.sqrt())
    }
}
