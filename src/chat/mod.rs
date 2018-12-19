//#[cfg(feature = "client")]
//pub use self::chat_gui::{ChatGui, ChatGuiAction};
pub use self::chat_msg::ChatMsg;
pub use self::chat_server::ChatServer;

//#[cfg(feature = "client")]
//pub mod chat_gui;
pub mod chat_msg;
pub mod chat_server;
