mod matchmaking;
mod rating;
mod update;

pub use rating::Rating;

use num_traits::Float;

#[derive(Debug, Clone, PartialEq)]
pub struct TrueSkill<F>
where
    F: Float,
{
    mu: F,
    sigma: F,
    beta: F,
    tau: F,
    draw_probability: F,
}

impl<F: Float> Eq for TrueSkill<F> {}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum Score {
    Win,
    Loss,
    Draw,
}

impl<F: Float> TrueSkill<F> {
    pub fn new(mu: F, sigma: F, beta: F, tau: F, draw_probability: F) -> Self {
        Self {
            mu,
            sigma,
            beta,
            tau,
            draw_probability,
        }
    }

    pub fn beta(&self) -> F {
        self.beta
    }

    pub fn tau(&self) -> F {
        self.tau
    }

    pub fn draw_probability(&self) -> F {
        self.draw_probability
    }

    pub fn create_rating(&self) -> Rating<F> {
        Rating::new(self.mu, self.sigma)
    }

    pub fn quality(&self, team1: &[Rating<F>], team2: &[Rating<F>]) -> F {
        matchmaking::quality(self, team1, team2)
    }

    pub fn balance(&self, players: &[Rating<F>]) -> (Vec<usize>, Vec<usize>) {
        matchmaking::balance(self, players)
    }

    pub fn update(
        &self,
        team1: &[Rating<F>],
        team2: &[Rating<F>],
        score: Score,
    ) -> (Vec<Rating<F>>, Vec<Rating<F>>) {
        update::update(self, team1, team2, score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<F: Float>() -> TrueSkill<F> {
        return TrueSkill::new(
            F::from(25).unwrap(),
            F::from(25f32 / 3f32).unwrap(),
            F::from(25f32 / 6f32).unwrap(),
            F::from(25f32 / 300f32).unwrap(),
            F::from(0.1).unwrap(),
        );
    }

    fn epsilon<F: Float>() -> F {
        F::from(0.001).unwrap()
    }

    fn check_rating<F: Float>(rating: Rating<F>, (mu, sigma): (F, F)) -> bool {
        (rating.mu() - mu).abs() < epsilon() && (rating.sigma() - sigma).abs() < epsilon()
    }

    #[test]
    fn quality() {
        let env = env();
        let team1 = [Rating::new(2.2, 1.7), Rating::new(36.7, 1.0)];
        let team2 = [Rating::new(20.3, 5.0), Rating::new(17.0, 7.3)];
        let quality = env.quality(&team1, &team2);
        assert!((quality - 0.671).abs() < epsilon())
    }

    #[test]
    fn balance() {
        let env = env();
        let players = [
            Rating::new(2.2, 1.7),
            Rating::new(17.0, 7.3),
            Rating::new(20.3, 5.0),
            Rating::new(36.7, 1.0),
        ];
        let teams = env.balance(&players);
        assert_eq!(teams.0, [0, 3]);
        assert_eq!(teams.1, [1, 2]);
    }

    #[test]
    fn update_win() {
        let env = env();
        let team1 = [Rating::new(2.2, 1.7), Rating::new(36.7, 1.0)];
        let team2 = [Rating::new(20.3, 5.0), Rating::new(17.0, 7.3)];
        let (new_team1, new_team2) = env.update(&team1, &team2, Score::Win);
        assert!(check_rating(new_team1[0], (2.381, 1.692)));
        assert!(check_rating(new_team1[1], (36.763, 1.001)));
        assert!(check_rating(new_team2[0], (18.737, 4.735)));
        assert!(check_rating(new_team2[1], (13.670, 6.447)));
    }

    #[test]
    fn update_loss() {
        let env = env();
        let team1 = [Rating::new(2.2, 1.7), Rating::new(36.7, 1.0)];
        let team2 = [Rating::new(20.3, 5.0), Rating::new(17.0, 7.3)];
        let (new_team1, new_team2) = env.update(&team1, &team2, Score::Loss);
        assert!(check_rating(new_team1[0], (1.979, 1.691)));
        assert!(check_rating(new_team1[1], (36.623, 1.001)));
        assert!(check_rating(new_team2[0], (22.208, 4.712)));
        assert!(check_rating(new_team2[1], (21.066, 6.367)));
    }

    #[test]
    fn update_draw() {
        let env = env();
        let team1 = [Rating::new(2.2, 1.7), Rating::new(36.7, 1.0)];
        let team2 = [Rating::new(20.3, 5.0), Rating::new(17.0, 7.3)];
        let (new_team1, new_team2) = env.update(&team1, &team2, Score::Draw);
        assert!(check_rating(new_team1[0], (2.170, 1.686)));
        assert!(check_rating(new_team1[1], (36.689, 1.000)));
        assert!(check_rating(new_team2[0], (20.563, 4.571)));
        assert!(check_rating(new_team2[1], (17.561, 5.883)));
    }
}
