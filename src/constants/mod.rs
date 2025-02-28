// constants.rs
#![allow(unused)]
use lazy_static::lazy_static;
use num::complex::Complex64;
pub const PI: f64 = std::f64::consts::PI;
pub const TOL: f64 = 1e-12;
pub const ZERO:Complex64 = Complex64::new(0.0, 0.0);
pub const ONE: Complex64 = Complex64::new(1.0, 0.0);
pub const HALF: Complex64 = Complex64::new(0.5, 0.0);
pub const MINUS_ONE: Complex64 = Complex64::new(-1.0, 0.0);
pub const INV_ROOT_TWO: Complex64 = Complex64::new(std::f64::consts::FRAC_1_SQRT_2, 0.0);
pub const IOTA: Complex64 = Complex64::new(0.0, 1.0);
pub const MINUS_IOTA: Complex64 = Complex64::new(0.0, -1.0);
pub const Z_PHASE: Complex64 = MINUS_ONE;
pub const  S_PHASE: Complex64 = IOTA;
// pub const TDG_PHASE: Complex64 = Complex64::from_polar(1.0, -PI / 4.0);
// pub const T_PHASE: Complex64= Complex64::from_polar(1.0, PI / 4.0);
pub const SDG_PHASE: Complex64 = Complex64::new(0.0, -1.0);
