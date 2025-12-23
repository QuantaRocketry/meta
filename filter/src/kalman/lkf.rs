// An implementation of a linear kalman filter

use nalgebra::{self, SMatrix, SVector};

pub struct LKF<const N: usize> {
    x: SVector<f32, N>,
    H: SMatrix<f32, N, N>,
}

impl<const N: usize> LKF<N> {
    pub fn new() {}

    pub fn predict(dt: f32) {}

    pub fn update(measurement: SVector<f32, N>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_adds_two() {
        assert_eq!(4, 2 + 2);
    }
}
