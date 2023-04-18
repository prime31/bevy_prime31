use bevy::prelude::*;
use rand::{distributions::Uniform, prelude::Distribution, Rng};

/// Samples uniformly from the surface of the unit sphere in three dimensions.
///
/// Implemented via a method by Marsaglia[^1].
///
///
/// # Example
///
/// ```
/// use rand_distr::{UnitSphere, Distribution};
///
/// let v: [f64; 3] = UnitSphere.sample(&mut rand::thread_rng());
/// println!("{:?} is from the unit sphere surface.", v)
/// ```

#[inline]
pub fn sample<R: Rng + ?Sized>(rng: &mut R) -> Vec3 {
    let uniform = Uniform::new(-1., 1.);
    loop {
        let (x1, x2) = (uniform.sample(rng), uniform.sample(rng));
        let sum: f32 = x1 * x1 + x2 * x2;
        if sum >= 1. {
            continue;
        }
        let factor = 2. * (1. - sum).sqrt();
        return Vec3::new(x1 * factor, x2 * factor, 1. - 2. * sum);
    }
}

#[inline]
pub fn sample_hemisphere<R: Rng + ?Sized>(rng: &mut R) -> Vec3 {
    let uniform = Uniform::new(-1., 1.);
    loop {
        let (x1, x2) = (uniform.sample(rng), uniform.sample(rng));
        let sum: f32 = x1 * x1 + x2 * x2;
        if sum >= 1. || x2 < 0. {
            continue;
        }
        let factor = 2. * (1. - sum).sqrt();
        return Vec3::new(x1 * factor, x2 * factor, 1. - 2. * sum);
    }
}
