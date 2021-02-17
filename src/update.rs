use super::{Rating, Score, TrueSkill};

use statrs::distribution::{Continuous, InverseCDF, Normal, Univariate};

pub fn update(
    env: &TrueSkill,
    team1: &[Rating],
    team2: &[Rating],
    score: Score,
) -> (Vec<Rating>, Vec<Rating>) {
    if score == Score::Loss {
        let (new_team2, new_team1) = update(env, team2, team1, Score::Win);
        return (new_team1, new_team2);
    }
    let player_count = team1.len() + team2.len();
    let tau = env.tau();
    let add_dynamic_factor = |x: &Rating| {
        let sigma = x.sigma();
        Rating::new(x.mu(), (sigma * sigma + tau * tau).sqrt())
    };
    let team1 = team1.iter().map(add_dynamic_factor).collect::<Vec<_>>();
    let team2 = team2.iter().map(add_dynamic_factor).collect::<Vec<_>>();
    let beta = env.beta();
    let epsilon = draw_margin(env.draw_probability(), player_count, beta);
    let player1: Rating = (&team1).into();
    let player2: Rating = (&team2).into();
    let sigma1 = player1.sigma();
    let sigma2 = player2.sigma();
    let c2 = player_count as f32 * beta * beta + sigma1 * sigma1 + sigma2 * sigma2;
    let c = c2.sqrt();
    let delta = player1.mu() - player2.mu();
    let (v, w) = {
        let t = delta / c;
        let epsilon = epsilon / c;
        if score == Score::Draw {
            (v_draw(t, epsilon), w_draw(t, epsilon))
        } else {
            (v(t, epsilon), w(t, epsilon))
        }
    };
    let f = |x: &Rating, mult: f32| {
        let sigma = x.sigma();
        let sigma2 = sigma * sigma;
        let mu = x.mu() + mult * sigma2 / c * v;
        let sigma = (sigma2 * (1. - sigma2 / c2 * w)).sqrt();
        Rating::new(mu, sigma)
    };
    let new_team1 = team1.iter().map(|x| f(x, 1.)).collect();
    let new_team2 = team2.iter().map(|x| f(x, -1.)).collect();
    (new_team1, new_team2)
}

fn v(t: f32, epsilon: f32) -> f32 {
    let normal = Normal::new(0f64, 1f64).unwrap();
    let x = t - epsilon;
    (normal.pdf(x as f64) / normal.cdf(x as f64)) as f32
}

fn v_draw(t: f32, epsilon: f32) -> f32 {
    let normal = Normal::new(0f64, 1f64).unwrap();
    let x1 = (-epsilon - t) as f64;
    let x2 = (epsilon - t) as f64;
    ((normal.pdf(x1) - normal.pdf(x2)) / (normal.cdf(x2) - normal.cdf(x1))) as f32
}

fn w(t: f32, epsilon: f32) -> f32 {
    v(t, epsilon) * (v(t, epsilon) + t - epsilon)
}

fn w_draw(t: f32, epsilon: f32) -> f32 {
    let normal = Normal::new(0f64, 1f64).unwrap();
    let v = v_draw(t, epsilon);
    let x1 = (epsilon - t) as f64;
    let x2 = (epsilon + t) as f64;
    v * v
        + ((x1 * normal.pdf(x1) + x2 * normal.pdf(x2)) / (normal.cdf(x1) - normal.cdf(-x2))) as f32
}

fn draw_margin(draw_probability: f32, n: usize, beta: f32) -> f32 {
    let normal = Normal::new(0f64, 1f64).unwrap();
    normal.inverse_cdf((1f64 + draw_probability as f64) / 2f64) as f32 * (n as f32).sqrt() * beta
}
