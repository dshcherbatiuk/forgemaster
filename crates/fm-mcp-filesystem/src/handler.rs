//! MCP handler exposing filesystem tools.

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData, tool, tool_handler, tool_router};

use crate::action;
use crate::param::{
    CreateDirectoryParams, DirectoryTreeParams, EditFileParams, ListDirectoryParams,
    ReadFileParams, SearchFilesParams, WriteFileParams,
};

/// MCP handler for filesystem operations.
#[derive(Clone)]
pub struct FilesystemHandler {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl FilesystemHandler {
    /// Creates a new handler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Read the contents of a text file. Returns the full file content as a string."
    )]
    async fn read_file(
        &self,
        Parameters(params): Parameters<ReadFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::read_file::execute(params).await)
    }

    #[tool(
        description = "Write content to a file. Creates the file and parent directories if they don't exist. Overwrites existing content."
    )]
    async fn write_file(
        &self,
        Parameters(params): Parameters<WriteFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::write_file::execute(params).await)
    }

    #[tool(
        description = "Edit a file by replacing the first occurrence of old_text with new_text. Fails if old_text is not found."
    )]
    async fn edit_file(
        &self,
        Parameters(params): Parameters<EditFileParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::edit_file::execute(params).await)
    }

    #[tool(description = "Create a directory and all parent directories (like mkdir -p).")]
    async fn create_directory(
        &self,
        Parameters(params): Parameters<CreateDirectoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::create_directory::execute(params).await)
    }

    #[tool(description = "List entries in a directory. Directories are suffixed with '/'.")]
    async fn list_directory(
        &self,
        Parameters(params): Parameters<ListDirectoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::list_directory::execute(params).await)
    }

    #[tool(
        description = "Generate a recursive directory tree with visual connectors (├── └──). Respects max_depth limit."
    )]
    async fn directory_tree(
        &self,
        Parameters(params): Parameters<DirectoryTreeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::directory_tree::execute(params).await)
    }

    #[tool(
        description = "Search file contents recursively by regex pattern. Returns matching lines with file path and line number."
    )]
    async fn search_files(
        &self,
        Parameters(params): Parameters<SearchFilesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(action::search_files::execute(params).await)
    }
}

impl Default for FilesystemHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for FilesystemHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "fm-mcp-filesystem".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Filesystem tools: read_file, write_file, edit_file, create_directory, list_directory, directory_tree, search_files".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_creation() {
        let handler = FilesystemHandler::new();
        let info = handler.get_info();
        assert_eq!(info.server_info.name, "fm-mcp-filesystem");
    }

    #[test]
    fn server_capabilities_has_tools() {
        let handler = FilesystemHandler::new();
        let info = handler.get_info();
        assert!(info.capabilities.tools.is_some());
    }

    #[test]
    fn instructions_mention_tools() {
        let handler = FilesystemHandler::new();
        let info = handler.get_info();
        let instructions = info.instructions.expect("should have instructions");
        assert!(instructions.contains("read_file"));
        assert!(instructions.contains("write_file"));
        assert!(instructions.contains("edit_file"));
        assert!(instructions.contains("create_directory"));
        assert!(instructions.contains("list_directory"));
        assert!(instructions.contains("directory_tree"));
        assert!(instructions.contains("search_files"));
    }
}
