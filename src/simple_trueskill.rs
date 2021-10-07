use std::iter::FromIterator;

use super::{Rating, Score};
use crate::utils::{cdf, pdf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleTrueSkill {
    mu: f64,
    sigma: f64,
    beta: f64,
    tau: f64,
}

impl Eq for SimpleTrueSkill {}

impl SimpleTrueSkill {
    #[inline]
    pub const fn new(mu: f64, sigma: f64, beta: f64, tau: f64) -> Self {
        Self {
            mu,
            sigma,
            beta,
            tau,
        }
    }

    #[inline]
    pub const fn mu(&self) -> f64 {
        self.mu
    }

    #[inline]
    pub const fn sigma(&self) -> f64 {
        self.sigma
    }

    #[inline]
    pub const fn beta(&self) -> f64 {
        self.beta
    }

    #[inline]
    pub const fn tau(&self) -> f64 {
        self.tau
    }

    #[inline]
    pub fn update(&self, team1: &mut [Rating], team2: &mut [Rating], score: Score) {
        if score == Score::Loss {
            self.update(team2, team1, Score::Win);
            return;
        }
        let player_count = (team1.len() + team2.len()) as f64;
        let add_dynamic_factor = |x: &mut Rating| *x += Rating::new(0.0, self.tau * self.tau);
        team1.iter_mut().for_each(add_dynamic_factor);
        team2.iter_mut().for_each(add_dynamic_factor);
        let rating1 = Rating::from_iter(team1.iter());
        let rating2 = Rating::from_iter(team2.iter());
        let c2 = player_count * self.beta * self.beta + rating1.variance() + rating2.variance();
        let c = c2.sqrt();
        let [v, w] = {
            let x = (rating1.mean() - rating2.mean()) / c;
            match score {
                Score::Win => Self::vw(x),
                Score::Draw => Self::vw_draw(x),
                Score::Loss => unreachable!(),
            }
        };
        let f = |x: &mut Rating, mult: f64| {
            *x = Rating::new(
                x.mean() + mult * x.variance() / c * v,
                x.variance() * (1.0 - x.variance() / c2 * w),
            )
        };
        team1.iter_mut().for_each(|x| f(x, 1.0));
        team2.iter_mut().for_each(|x| f(x, -1.0));
    }

    #[inline]
    pub fn create_rating(&self) -> Rating {
        Rating::new(self.mu, self.sigma * self.sigma)
    }

    #[inline]
    pub fn quality(&self, team1: &[Rating], team2: &[Rating]) -> f64 {
        let player1 = Rating::from_iter(team1);
        let player2 = Rating::from_iter(team2);
        let n = team1.len() + team2.len();
        let dmu = player1.mean() - player2.mean();
        let nb2 = (n as f64) * self.beta * self.beta;
        let c2 = nb2 + player1.variance() + player2.variance();
        let u = (nb2 / c2).sqrt();
        let v = -dmu * dmu / (2.0 * c2);
        u * v.exp()
    }

    #[inline]
    fn vw(x: f64) -> [f64; 2] {
        let v = pdf(x) / cdf(x);
        let w = v * (v + x);
        [v, w]
    }

    #[inline]
    fn vw_draw(x: f64) -> [f64; 2] {
        [-x, 1.0]
    }
}

#[cfg(test)]
mod test {
    use statrs::assert_almost_eq;

    use super::*;

    static MU: f64 = 3.0;
    static SIGMA: f64 = 1.0;
    static BETA: f64 = 0.5;
    static TAU: f64 = 0.1;
    static TRUESKILL: SimpleTrueSkill = SimpleTrueSkill::new(MU, SIGMA, BETA, TAU);

    static TEAM1: [Rating; 2] = [Rating::new(1.0, 0.1), Rating::new(4.0, 0.5)];
    static TEAM2: [Rating; 2] = [Rating::new(2.0, 0.3), Rating::new(2.5, 0.7)];

    #[test]
    fn new() {
        assert_almost_eq!(TRUESKILL.mu(), MU, 1e-15);
        assert_almost_eq!(TRUESKILL.sigma(), SIGMA, 1e-15);
        assert_almost_eq!(TRUESKILL.beta(), BETA, 1e-15);
        assert_almost_eq!(TRUESKILL.tau(), TAU, 1e-15);
    }

    #[test]
    fn create_rating() {
        let rating = TRUESKILL.create_rating();
        assert_almost_eq!(rating.mean(), MU, 1e-15);
        assert_almost_eq!(rating.variance(), SIGMA * SIGMA, 1e-15);
    }

    #[test]
    fn update_win() {
        let mut team1 = TEAM1;
        let mut team2 = TEAM2;
        TRUESKILL.update(&mut team1, &mut team2, Score::Win);
        assert_almost_eq!(team1[0].mean(), 1.0414903391243753, 1e-15);
        assert_almost_eq!(team1[1].mean(), 4.192364299576649, 1e-15);
        assert_almost_eq!(team2[0].mean(), 1.8830726806494875, 1e-15);
        assert_almost_eq!(team2[1].mean(), 2.2321987201972133, 1e-15);
        assert_almost_eq!(team1[0].variance(), 0.10741416969425319, 1e-15);
        assert_almost_eq!(team1[1].variance(), 0.4544153336756405, 1e-15);
        assert_almost_eq!(team2[0].variance(), 0.2894629510427876, 1e-15);
        assert_almost_eq!(team2[1].variance(), 0.6022713175928119, 1e-15);
    }

    #[test]
    fn update_draw() {
        let mut team1 = TEAM1;
        let mut team2 = TEAM2;
        TRUESKILL.update(&mut team1, &mut team2, Score::Draw);
        assert_almost_eq!(team1[0].mean(), 0.9791666666666666, 1e-15);
        assert_almost_eq!(team1[1].mean(), 3.903409090909091, 1e-15);
        assert_almost_eq!(team2[0].mean(), 2.058712121212121, 1e-15);
        assert_almost_eq!(team2[1].mean(), 2.634469696969697, 1e-15);
        assert_almost_eq!(team1[0].variance(), 0.10541666666666669, 1e-15);
        assert_almost_eq!(team1[1].variance(), 0.41147727272727275, 1e-15);
        assert_almost_eq!(team2[0].variance(), 0.27359848484848487, 1e-15);
        assert_almost_eq!(team2[1].variance(), 0.5190530303030303, 1e-15);
    }

    #[test]
    fn quality() {
        let quality = TRUESKILL.quality(&TEAM1, &TEAM2);
        assert_almost_eq!(quality, 0.5910630134064284, 1e-15);
    }
}
