/**
 * AddAgentDialog test specification (manual/visual testing)
 *
 * These describe the expected behavior. Run the app and verify:
 *
 * 1. Dialog opens with empty fields
 *    - Name input is focused
 *    - Config path input is empty
 *    - Config format defaults to "json"
 *    - MCP key defaults to "mcpServers"
 *    - "Add" button is disabled when name is empty
 *
 * 2. "Add" button enables when name AND config path are filled
 *    - Name: "My Agent"
 *    - Config path: "/path/to/config.json"
 *    - Button should be enabled
 *
 * 3. "Add" button calls addCustomAgent with correct params
 *    - name: "My Agent"
 *    - configPath: "/path/to/config.json"
 *    - configFormat: "json"
 *    - mcpKey: "mcpServers"
 *    - After success: dialog closes, onAdded() is called
 *
 * 4. Error handling
 *    - If addCustomAgent fails, show error toast
 *    - Dialog stays open on error
 *
 * 5. JSON snippet shown for reference
 *    - Shows plugmux config to add to the agent
 *    - Copy button works
 *
 * 6. Config format selector
 *    - "json" and "toml" options
 *    - Changing format updates the MCP key suggestion
 *      json -> "mcpServers", toml -> "mcp_servers"
 */
export {};
