use std::fmt::Debug;
use std::ops::AddAssign;

use num::One;

/// We use the weight to pick a production rule at random.
pub trait Weight: One + AddAssign + Copy + Default + Into<f64> + Debug + 'static {}

impl Weight for u8 {}
impl Weight for u16 {}
impl Weight for u32 {}
impl Weight for f32 {}
impl Weight for f64 {}
