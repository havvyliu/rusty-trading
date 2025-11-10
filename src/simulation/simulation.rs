use rand::prelude::*;
use statrs::distribution::{Normal};

pub fn henson(mut price: f64) -> f64 {
    let mut v = 0.04;
    let mu = 0.10;
    let kappa = 2.0;
    let theta = 0.04;
    let xi = 0.8; // Added missing semicolon
    let rho = -0.6; // Added missing semicolon
    let dt = 1.0 / 252.0 / 24.0 / 60.0/ 60.0;
    let v_min = 1e-6;

    // 1. correlated normals
    let mut rng = rand::thread_rng();
    let z1 = rng.sample(Normal::new(0.0, 1.0).unwrap());
    let z2 = rng.sample(Normal::new(0.0 ,1.0).unwrap());
    
    let eps1 = z1;
    let eps2 = rho * z1 + (1 as f64 - rho*rho).sqrt() * z2;

    // // 2. update variance (Euler)
    v = v + kappa * (theta - v) * dt
        + xi * ((v as f64).max(v_min)).sqrt() * (dt as f64).sqrt() * eps2;

    // // floor it
    if v < v_min {
        v = v_min;
    }

    // // 3. update price (GBM w/ stochastic vol)
    price = price * ((mu - 0.5 * v).exp() * dt + v.sqrt() * dt.sqrt() * eps1);

    price
}