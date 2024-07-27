//! Classification of rules and grammars.

// mod linear;
// mod recursive;
// pub mod cyclical;
// #[cfg(feature = "cfg-predict-sets")]
// pub mod ll;
// pub mod lr;

#[cfg(feature = "cyclical")]
pub use cfg_classify_cyclical::*;
// #[cfg(feature = "linear")]
// pub use cfg_classify_linear::*;
#[cfg(feature = "ll")]
pub use cfg_classify_ll::*;
#[cfg(feature = "lr")]
pub use cfg_classify_lr::*;
#[cfg(feature = "useful")]
pub use cfg_classify_useful::*;
