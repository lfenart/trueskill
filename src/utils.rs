use core::f64;
use statrs::function::erf;

#[inline]
pub fn cdf(x: f64) -> f64 {
    0.5 * erf::erfc(-x / f64::consts::SQRT_2)
}

#[inline]
pub fn pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / statrs::consts::SQRT_2PI
}

#[inline]
pub fn inverse_cdf(x: f64) -> f64 {
    -f64::consts::SQRT_2 * erf::erfc_inv(2.0 * x)
}
