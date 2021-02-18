use super::{Rating, Score, TrueSkill};

use num_traits::Float;
use statrs::distribution::{Continuous, InverseCDF, Normal, Univariate};

pub fn update<F: Float>(
    env: &TrueSkill<F>,
    team1: &[Rating<F>],
    team2: &[Rating<F>],
    score: Score,
) -> (Vec<Rating<F>>, Vec<Rating<F>>) {
    if score == Score::Loss {
        let (new_team2, new_team1) = update(env, team2, team1, Score::Win);
        return (new_team1, new_team2);
    }
    let player_count = F::from(team1.len() + team2.len()).unwrap();
    let tau = env.tau();
    let add_dynamic_factor = |x: &Rating<F>| {
        let sigma = x.sigma();
        Rating::new(x.mu(), (sigma * sigma + tau * tau).sqrt())
    };
    let team1 = team1.iter().map(add_dynamic_factor).collect::<Vec<_>>();
    let team2 = team2.iter().map(add_dynamic_factor).collect::<Vec<_>>();
    let beta = env.beta();
    let epsilon = draw_margin(env.draw_probability(), player_count, beta);
    let player1: Rating<F> = (&team1).into();
    let player2: Rating<F> = (&team2).into();
    let sigma1 = player1.sigma();
    let sigma2 = player2.sigma();
    let c2 = player_count * beta * beta + sigma1 * sigma1 + sigma2 * sigma2;
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
    let f = |x: &Rating<F>, mult: F| {
        let sigma = x.sigma();
        let sigma2 = sigma * sigma;
        let new_mu = x.mu() + mult * sigma2 / c * v;
        let new_sigma = (sigma2 * (F::one() - sigma2 / c2 * w)).sqrt();
        Rating::new(new_mu, new_sigma)
    };
    let new_team1 = team1.iter().map(|x| f(x, F::one())).collect();
    let new_team2 = team2.iter().map(|x| f(x, -F::one())).collect();
    (new_team1, new_team2)
}

fn v<F: Float>(t: F, epsilon: F) -> F {
    let normal = Normal::new(0., 1.).unwrap();
    let x = (t - epsilon).to_f64().unwrap();
    F::from(normal.pdf(x) / normal.cdf(x)).unwrap()
}

fn v_draw<F: Float>(t: F, epsilon: F) -> F {
    let normal = Normal::new(0., 1.).unwrap();
    let x1 = (-epsilon - t).to_f64().unwrap();
    let x2 = (epsilon - t).to_f64().unwrap();
    F::from((normal.pdf(x1) - normal.pdf(x2)) / (normal.cdf(x2) - normal.cdf(x1))).unwrap()
}

fn w<F: Float>(t: F, epsilon: F) -> F {
    v(t, epsilon) * (v(t, epsilon) + t - epsilon)
}

fn w_draw<F: Float>(t: F, epsilon: F) -> F {
    let normal = Normal::new(0., 1.).unwrap();
    let v = v_draw(t, epsilon);
    let x1 = (epsilon - t).to_f64().unwrap();
    let x2 = (epsilon + t).to_f64().unwrap();
    v * v
        + F::from(
            x1.mul_add(normal.pdf(x1), x2 * normal.pdf(x2)) / (normal.cdf(x1) - normal.cdf(-x2)),
        )
        .unwrap()
}

fn draw_margin<F: Float>(draw_probability: F, n: F, beta: F) -> F {
    let normal = Normal::new(0., 1.).unwrap();
    F::from(normal.inverse_cdf((1. + draw_probability.to_f64().unwrap()) / 2.)).unwrap()
        * n.sqrt()
        * beta
}
