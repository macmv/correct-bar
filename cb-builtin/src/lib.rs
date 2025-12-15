macro_rules! feature_mod {
  ($mod:ident, $feature:literal) => {
    #[cfg(feature = $feature)]
    pub mod $mod;

    #[cfg(feature = $feature)]
    pub use $mod::*;
  };
}

feature_mod!(clock, "clock");
feature_mod!(hwmon, "hwmon");
feature_mod!(hypr, "hypr");
feature_mod!(proc, "proc");
