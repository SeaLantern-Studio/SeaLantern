mod definitions;
mod filename;
mod main_class;

pub use definitions::CoreType;
pub use filename::detect_core_type;
pub(super) use main_class::detect_core_type_with_main_class;
