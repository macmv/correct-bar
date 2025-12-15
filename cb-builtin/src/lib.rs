macro_rules! feature_mod {
  ($mod:ident, $feature:literal) => {
    #[cfg(feature = $feature)]
    pub mod $mod;

    #[cfg(feature = $feature)]
    pub use $mod::*;
  };
}

feature_mod!(clock, "clock");
