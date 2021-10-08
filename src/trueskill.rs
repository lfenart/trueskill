use core::iter::FromIterator;

use super::{Rating, Score};
use crate::utils::{cdf, inverse_cdf, pdf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrueSkill {
    mu: f64,
    sigma: f64,
    beta: f64,
    tau: f64,
    draw_probability: f64,
}

impl Eq for TrueSkill {}

impl TrueSkill {
    #[inline]
    pub const fn new(mu: f64, sigma: f64, beta: f64, tau: f64, draw_probability: f64) -> Self {
        Self {
            mu,
            sigma,
            beta,
            tau,
            draw_probability,
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
    pub const fn draw_probability(&self) -> f64 {
        self.draw_probability
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
            let t = (rating1.mean() - rating2.mean()) / c;
            let epsilon = Self::draw_margin(self.draw_probability(), player_count, self.beta) / c;
            match score {
                Score::Win => Self::vw(t, epsilon),
                Score::Draw => Self::vw_draw(t, epsilon),
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
    fn vw(t: f64, epsilon: f64) -> [f64; 2] {
        let x = t - epsilon;
        let v = pdf(x) / cdf(x);
        let w = v * (v + x);
        [v, w]
    }

    #[inline]
    fn vw_draw(t: f64, epsilon: f64) -> [f64; 2] {
        let x1 = -epsilon - t;
        let x2 = epsilon - t;
        let den = cdf(x2) - cdf(x1);
        let v = (pdf(x1) - pdf(x2)) / den;
        let w = v * v + (x2 * pdf(x2) - x1 * pdf(-x1)) / den;
        [v, w]
    }

    #[inline]
    fn draw_margin(draw_probability: f64, n: f64, beta: f64) -> f64 {
        inverse_cdf((1.0 + draw_probability) / 2.0) * n.sqrt() * beta
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
    static DRAW_PROBABILITY: f64 = 0.1;
    static TRUESKILL: TrueSkill = TrueSkill::new(MU, SIGMA, BETA, TAU, DRAW_PROBABILITY);

    static TEAM1: [Rating; 2] = [Rating::new(1.0, 0.1), Rating::new(4.0, 0.5)];
    static TEAM2: [Rating; 2] = [Rating::new(2.0, 0.3), Rating::new(2.5, 0.7)];

    #[test]
    fn new() {
        assert_almost_eq!(TRUESKILL.mu(), MU, 1e-15);
        assert_almost_eq!(TRUESKILL.sigma(), SIGMA, 1e-15);
        assert_almost_eq!(TRUESKILL.beta(), BETA, 1e-15);
        assert_almost_eq!(TRUESKILL.tau(), TAU, 1e-15);
        assert_almost_eq!(TRUESKILL.draw_probability(), DRAW_PROBABILITY, 1e-15);
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
        assert_almost_eq!(team1[0].mean(), 1.044494855140872, 1e-15);
        assert_almost_eq!(team1[1].mean(), 4.206294328380406, 1e-15);
        assert_almost_eq!(team2[0].mean(), 1.8746054082393613, 1e-15);
        assert_almost_eq!(team2[1].mean(), 2.2128059349998273, 1e-15);
        assert_almost_eq!(team1[0].variance(), 0.10732620185993816, 1e-15);
        assert_almost_eq!(team1[1].variance(), 0.45252438874131495, 1e-15);
        assert_almost_eq!(team2[0].variance(), 0.2887642974165335, 1e-15);
        assert_almost_eq!(team2[1].variance(), 0.5986064758342824, 1e-15);
    }

    #[test]
    fn update_draw() {
        let mut team1 = TEAM1;
        let mut team2 = TEAM2;
        TRUESKILL.update(&mut team1, &mut team2, Score::Draw);
        assert_almost_eq!(team1[0].mean(), 0.9792081691641251, 1e-15);
        assert_almost_eq!(team1[1].mean(), 3.903601511579126, 1e-15);
        assert_almost_eq!(team2[0].mean(), 2.0585951596283745, 1e-15);
        assert_almost_eq!(team2[1].mean(), 2.634201817213374, 1e-15);
        assert_almost_eq!(team1[0].variance(), 0.10542579652761795, 1e-15);
        assert_almost_eq!(team1[1].variance(), 0.411673527011027, 1e-15);
        assert_almost_eq!(team2[0].variance(), 0.273670995562321, 1e-15);
        assert_almost_eq!(team2[1].variance(), 0.5194333908737359, 1e-15);
    }

    #[test]
    fn update_loss() {
        let mut team1 = TEAM1;
        let mut team2 = TEAM2;
        TRUESKILL.update(&mut team1, &mut team2, Score::Loss);
        assert_almost_eq!(team1[0].mean(), 0.9283660862362771, 1e-15);
        assert_almost_eq!(team1[1].mean(), 3.6678791270954667, 1e-15);
        assert_almost_eq!(team2[0].mean(), 2.201877393334128, 1e-15);
        assert_almost_eq!(team2[1].mean(), 2.9623643524749386, 1e-15);
        assert_almost_eq!(team1[0].variance(), 0.1067360228558045, 1e-15);
        assert_almost_eq!(team1[1].variance(), 0.43983797890865656, 1e-15);
        assert_almost_eq!(team2[0].variance(), 0.2840770079704802, 1e-15);
        assert_almost_eq!(team2[1].variance(), 0.5740189356703336, 1e-15);
    }

    #[test]
    fn quality() {
        let quality = TRUESKILL.quality(&TEAM1, &TEAM2);
        assert_almost_eq!(quality, 0.5910630134064284, 1e-15);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        let text = serde_json::to_string(&TRUESKILL).unwrap();
        assert_eq!(
            text,
            r#"{"mu":3.0,"sigma":1.0,"beta":0.5,"tau":0.1,"draw_probability":0.1}"#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize() {
        let trueskill = serde_json::from_str::<TrueSkill>(
            r#"{"mu":3.0,"sigma":1.0,"beta":0.5,"tau":0.1,"draw_probability":0.1}"#,
        )
        .unwrap();
        assert_eq!(trueskill, TRUESKILL);
    }
}
