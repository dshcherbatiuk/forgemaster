//! WebSocket module for real-time UI communication.

mod action;
mod command;
mod connection_registry;
mod event;
mod handler;
mod router;
mod schema_cache;
mod server;
mod task_creator;

pub use action::{ActionDispatcher, WsAction};
pub use command::WsCommand;
pub use connection_registry::ConnectionRegistry;
pub use event::WsEvent;
pub use schema_cache::SchemaCache;
pub use server::WsServer;
