//! WebSocket module for real-time UI communication.

mod action;
mod active_task_store;
mod command;
mod connection_registry;
mod event;
mod handler;
mod router;
mod schema_cache;
mod server;
mod task_creator;
mod task_state_broadcaster;
mod task_status_schema;

pub use action::{ActionDispatcher, WsAction};
pub use active_task_store::ActiveTaskStore;
pub use command::WsCommand;
pub use connection_registry::ConnectionRegistry;
pub use event::WsEvent;
pub use schema_cache::SchemaCache;
pub use server::WsServer;
pub use task_state_broadcaster::TaskStateBroadcaster;
