use crate::{Rating, TrueSkill};

use itertools::Itertools;
use num_traits::Float;

pub fn quality<F: Float>(env: &TrueSkill<F>, team1: &[Rating<F>], team2: &[Rating<F>]) -> F {
    let player1: Rating<F> = team1.into();
    let player2: Rating<F> = team2.into();
    let beta = env.beta();
    let n = team1.len() + team2.len();
    let nb2 = F::from(n).unwrap() * beta * beta;
    let sigma1 = player1.sigma();
    let sigma2 = player2.sigma();
    let dmu = player1.mu() - player2.mu();
    let c2 = nb2 + sigma1 * sigma1 + sigma2 * sigma2;
    let u = (nb2 / c2).sqrt();
    let v = (-(dmu * dmu) / (F::from(2).unwrap() * c2)).exp();
    u * v
}

pub fn balance<F: Float>(env: &TrueSkill<F>, players: &[Rating<F>]) -> (Vec<usize>, Vec<usize>) {
    let mut best_quality = F::zero();
    let mut best_teams = None;
    let len = players.len();
    for v in (1..len).combinations(len / 2) {
        let mut is_team1 = vec![true; len];
        for i in v {
            is_team1[i] = false;
        }
        let mut team1 = Vec::new();
        let mut team2 = Vec::new();
        for (i, &check) in is_team1.iter().enumerate() {
            if check { &mut team1 } else { &mut team2 }.push(i);
        }
        let quality = env.quality(
            &team1.iter().map(|&x| players[x]).collect::<Vec<_>>(),
            &team2.iter().map(|&x| players[x]).collect::<Vec<_>>(),
        );
        if quality > best_quality {
            best_quality = quality;
            best_teams = Some((team1, team2));
        }
    }
    best_teams.unwrap()
}
