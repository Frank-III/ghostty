//! Deep equality helpers ported from `src/datastruct/comparison.zig`.

/// Types that can participate in recursive deep equality checks.
pub trait DeepEqual {
    fn deep_equal(&self, other: &Self) -> bool;
}

impl DeepEqual for () {
    fn deep_equal(&self, _other: &Self) -> bool {
        true
    }
}

impl DeepEqual for bool {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

macro_rules! impl_deep_equal_primitive {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl DeepEqual for $ty {
                fn deep_equal(&self, other: &Self) -> bool {
                    self == other
                }
            }
        )+
    };
}

impl_deep_equal_primitive!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

impl DeepEqual for str {
    fn deep_equal(&self, other: &Self) -> bool {
        self == other
    }
}

impl DeepEqual for String {
    fn deep_equal(&self, other: &Self) -> bool {
        self.as_str().deep_equal(other.as_str())
    }
}

impl DeepEqual for &str {
    fn deep_equal(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl<T: DeepEqual> DeepEqual for Option<T> {
    fn deep_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (None, None) => true,
            (Some(a), Some(b)) => a.deep_equal(b),
            _ => false,
        }
    }
}

impl<T: DeepEqual> DeepEqual for [T] {
    fn deep_equal(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(a, b)| a.deep_equal(b))
    }
}

impl<T: DeepEqual, const N: usize> DeepEqual for [T; N] {
    fn deep_equal(&self, other: &Self) -> bool {
        self.as_slice().deep_equal(other.as_slice())
    }
}

impl<T: DeepEqual> DeepEqual for Vec<T> {
    fn deep_equal(&self, other: &Self) -> bool {
        self.as_slice().deep_equal(other.as_slice())
    }
}

/// Compare two values using [`DeepEqual`].
pub fn deep_equal<T: DeepEqual + ?Sized>(old: &T, new: &T) -> bool {
    old.deep_equal(new)
}

/// Approximate float equality using `sqrt(eps)` relative tolerance, mirroring Zig.
pub fn approx_eq_f32(expected: f32, actual: f32) -> bool {
    approx_eq_f64_tol(expected as f64, actual as f64, f32::EPSILON.sqrt() as f64)
}

pub fn approx_eq_f64(expected: f64, actual: f64) -> bool {
    approx_eq_f64_tol(expected, actual, f64::EPSILON.sqrt())
}

fn approx_eq_f64_tol(expected: f64, actual: f64, tolerance: f64) -> bool {
    if expected == actual {
        return true;
    }
    let diff = (expected - actual).abs();
    let scale = expected.abs().max(actual.abs());
    if scale == 0.0 {
        return diff <= tolerance;
    }
    diff / scale <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_equal_primitives() {
        assert!(deep_equal(&42i32, &42));
        assert!(!deep_equal(&42i32, &43));
        assert!(deep_equal("hello", "hello"));
        assert!(!deep_equal("hello", "world"));
    }

    #[test]
    fn deep_equal_option_and_array() {
        assert!(deep_equal(&Some(1i32), &Some(1)));
        assert!(!deep_equal(&Some(1i32), &None));
        assert!(deep_equal(&[1, 2, 3], &[1, 2, 3]));
        assert!(!deep_equal(&[1, 2, 3], &[1, 2, 4]));
    }

    #[test]
    fn approx_equal_floats() {
        assert!(approx_eq_f32(10.0, 10.000001));
        let tol = f64::EPSILON.sqrt();
        assert!(approx_eq_f64(4.0, 4.0 * (1.0 + tol * 0.5)));
    }
}
