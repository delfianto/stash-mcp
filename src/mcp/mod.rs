pub mod prompts;
pub mod resources;
pub mod tools;

use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, GetPromptRequestParams, GetPromptResult,
        Implementation, ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult,
        ListToolsResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
};

use crate::{config::Config, stash::StashClient};

pub struct StashMcpHandler {
    client: Arc<StashClient>,
    config: Arc<Config>,
}

impl StashMcpHandler {
    pub fn new(client: StashClient, config: Config) -> Self {
        Self {
            client: Arc::new(client),
            config: Arc::new(config),
        }
    }
}

impl ServerHandler for StashMcpHandler {
    fn get_info(&self) -> ServerInfo {
        let mut caps = ServerCapabilities::default();
        caps.tools = Some(Default::default());
        caps.resources = Some(Default::default());
        caps.prompts = Some(Default::default());

        let mut info = ServerInfo::default();
        info.protocol_version = rmcp::model::ProtocolVersion::LATEST;
        info.capabilities = caps;
        let mut impl_info = Implementation::default();
        impl_info.name = "stash-mcp".to_owned();
        impl_info.version = env!("CARGO_PKG_VERSION").to_owned();
        info.server_info = impl_info;
        info.instructions = Some(
            "MCP server for StashApp. \
             Use the tools to query performers, scenes, studios and tags. \
             Use the resources to browse collections. \
             Use the prompts as starting points for analysis tasks."
                .to_owned(),
        );
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(tools::list_tools())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        tools::call_tool(&self.client, &self.config, request).await
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(resources::list_resources())
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(resources::list_resource_templates())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        resources::read_resource(&self.client, &self.config, request).await
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(prompts::list_prompts())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        prompts::get_prompt(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::collections::HashMap;

    fn make_handler() -> StashMcpHandler {
        let config = Config::from_map(HashMap::new()).unwrap();
        let client = StashClient::new(&config);
        StashMcpHandler::new(client, config)
    }

    #[test]
    fn get_info_has_correct_server_name() {
        let handler = make_handler();
        let info = handler.get_info();
        assert_eq!(info.server_info.name, "stash-mcp");
    }

    #[test]
    fn get_info_advertises_tools_resources_prompts() {
        let handler = make_handler();
        let info = handler.get_info();
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.resources.is_some());
        assert!(info.capabilities.prompts.is_some());
    }

    #[test]
    fn get_info_has_instructions() {
        let handler = make_handler();
        let info = handler.get_info();
        assert!(info.instructions.is_some());
    }
}
