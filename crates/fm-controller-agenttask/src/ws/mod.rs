//! WebSocket module for real-time UI communication.

mod command;
mod connection_registry;
mod event;
mod handler;
mod router;
mod schema_cache;
mod server;

pub use command::WsCommand;
pub use connection_registry::ConnectionRegistry;
pub use event::WsEvent;
pub use schema_cache::SchemaCache;
pub use server::WsServer;
