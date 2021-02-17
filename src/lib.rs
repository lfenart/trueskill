mod matchmaking;
mod rating;
mod update;

pub use rating::Rating;

#[derive(Debug, Clone, PartialEq)]
pub struct TrueSkill {
    mu: f32,
    sigma: f32,
    beta: f32,
    tau: f32,
    draw_probability: f32,
}

impl Eq for TrueSkill {}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum Score {
    Win,
    Loss,
    Draw,
}

impl TrueSkill {
    pub const fn new(mu: f32, sigma: f32, beta: f32, tau: f32, draw_probability: f32) -> Self {
        Self {
            mu,
            sigma,
            beta,
            tau,
            draw_probability,
        }
    }

    pub const fn beta(&self) -> f32 {
        self.beta
    }

    pub const fn tau(&self) -> f32 {
        self.tau
    }

    pub const fn draw_probability(&self) -> f32 {
        self.draw_probability
    }

    pub const fn create_rating(&self) -> Rating {
        Rating::new(self.mu, self.sigma)
    }

    pub fn quality(&self, team1: &[Rating], team2: &[Rating]) -> f32 {
        matchmaking::quality(self, team1, team2)
    }

    pub fn balance(&self, players: &[Rating]) -> (Vec<usize>, Vec<usize>) {
        matchmaking::balance(self, players)
    }

    pub fn update(
        &self,
        team1: &[Rating],
        team2: &[Rating],
        score: Score,
    ) -> (Vec<Rating>, Vec<Rating>) {
        update::update(self, team1, team2, score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> TrueSkill {
        return TrueSkill::new(25., 25. / 3., 25. / 6., 25. / 300., 0.1);
    }

    fn check_rating(rating: Rating, (mu, sigma): (f32, f32)) -> bool {
        (rating.mu() - mu).abs() < 0.001 && (rating.sigma() - sigma).abs() < 0.001
    }

    #[test]
    fn quality() {
        let env = env();
        let team1 = [Rating::new(2.2, 1.7), Rating::new(36.7, 1.0)];
        let team2 = [Rating::new(20.3, 5.0), Rating::new(17.0, 7.3)];
        let quality = env.quality(&team1, &team2);
        assert!((quality - 0.671).abs() < 0.001)
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
