//! Linear interpolation

/// Performs linear interpolation.
/// A linear interpolation consists of two states 'a' and 'b'.
/// The 't' variable is a factor between 0 and 1 that
/// gives weight to 'a' or 'b'.
/// When 't' is zero then 'a' has full weight.
/// When 't' is one then 'b' has full weight.
#[inline(always)]
#[allow(dead_code)]
pub fn lerp<T: Lerp>(a: &T, b: &T, t: &T::Scalar) -> T {
    a.lerp(b, t)
}

/// Describes a type that can linearly interpolate between two points.
pub trait Lerp {
    /// The scaling type for linear interpolation.
    type Scalar;

    /// Given `self` and another point `other`, return a point on a line running between the two
    /// that is `scalar` fraction of the distance between the two points.
    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self;
}

/// Implementation of `Lerp` for floats.
macro_rules! impl_lerp_for_float {
    ($float: ident) => {
        impl Lerp for $float {
            type Scalar = $float;

            #[inline(always)]
            fn lerp(&self, other: &$float, scalar: &$float) -> $float {
                self + (other - self) * scalar
            }
        }
    };
}

impl_lerp_for_float!(f32);
