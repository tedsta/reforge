#[cfg(feature = "client")]
pub use self::star_map_gui::{StarMapGui, StarMapGuiAction};
pub use self::star_map_server::{StarMapAction, StarMapServer};

#[cfg(feature = "client")]
pub mod star_map_gui;
pub mod star_map_server;
pub mod station;