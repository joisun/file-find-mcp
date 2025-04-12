use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo};
use rmcp::{ServerHandler, schemars, tool};
use std::fs;
use std::path::Path;
use std::process::Command;
use grep::regex::RegexMatcher;
use grep::searcher::{BinaryDetection, SearcherBuilder};
use ignore::WalkBuilder;
use tracing;

// Search parameters: directory path and search keyword
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    #[schemars(description = "Path to the directory to search")]
    pub directory: String,
    #[schemars(description = "Keyword to search for")]
    pub keyword: String,
}

// File content parameters: file path
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FileContentParams {
    #[schemars(description = "Path to the file to read")]
    pub file_path: String,
}

// Main tool struct
#[derive(Debug, Clone)]
pub struct SearchTool;

#[tool(tool_box)]
impl SearchTool {
    pub fn new() -> Self {
        Self {}
    }

    /// Read and return the content of a specified file
    #[tool(description = "Read the content of a file from the specified path")]
    async fn read_file_content(
        &self,
        #[tool(aggr)] params: FileContentParams,
    ) -> Result<String, String> {
        // Validate file path
        let file_path = Path::new(&params.file_path);

        // Check if the path exists
        if !file_path.exists() {
            return Err(format!(
                "The specified path '{}' does not exist",
                params.file_path
            ));
        }

        // Check if the path is a file
        if !file_path.is_file() {
            return Err(format!(
                "The specified path '{}' is not a file",
                params.file_path
            ));
        }

        // Try to read the file content
        match fs::read_to_string(file_path) {
            Ok(content) => {
                if content.is_empty() {
                    Ok("File is empty.".to_string())
                } else {
                    Ok(content)
                }
            }
            Err(e) => {
                // Handle binary files or read errors
                tracing::error!("Error reading file '{}': {}", file_path.display(), e);

                // Try to read as binary and check if it's a binary file
                match fs::read(file_path) {
                    Ok(bytes) => {
                        // Check if it seems to be a binary file
                        if bytes.iter().any(|&b| b == 0)
                            || bytes
                                .iter()
                                .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
                                .count()
                                > bytes.len() / 10
                        {
                            Err(format!(
                                "The file '{}' appears to be a binary file and cannot be displayed as text",
                                params.file_path
                            ))
                        } else {
                            Err(format!(
                                "The file '{}' could not be read as text: {}",
                                params.file_path, e
                            ))
                        }
                    }
                    Err(read_err) => Err(format!(
                        "Error reading file '{}': {}",
                        params.file_path, read_err
                    )),
                }
            }
        }
    }

    /// Perform fast search for keywords in files using ripgrep
    #[tool(description = "Search for keywords in text files within the specified directory")]
    async fn search(&self, #[tool(aggr)] params: SearchParams) -> Result<String, String> {
        // Validate directory path
        let dir_path = Path::new(&params.directory);
        if !dir_path.exists() {
            return Err(format!(
                "The specified path '{}' does not exist",
                params.directory
            ));
        }

        if !dir_path.is_dir() {
            return Err(format!(
                "The specified path '{}' is not a directory",
                params.directory
            ));
        }

        // Ensure the keyword is not empty
        if params.keyword.trim().is_empty() {
            return Err("Search keyword is empty. Please enter a valid keyword.".into());
        }

        tracing::info!("Starting search for '{}' in {}", params.keyword, params.directory);

        // Method 1: Use ripgrep directly through process
        let results = self.search_with_ripgrep(&params.directory, &params.keyword)?;
        
        if results.is_empty() {
            Ok(format!(
                "No search results for keyword '{}'.",
                params.keyword
            ))
        } else {
            Ok(format!(
                "Search results:\n{}",
                results
            ))
        }
    }

    // Helper method to search using ripgrep process
    fn search_with_ripgrep(&self, directory: &str, keyword: &str) -> Result<String, String> {
        // Run ripgrep command
        let output = Command::new("rg")
            .arg("--json")           // Output in JSON format
            .arg("--max-count=10")   // Limit to 10 matches per file
            .arg("--max-depth=10")   // Limit directory depth
            .arg("--ignore-case")    // Case insensitive search
            .arg("--no-ignore")      // Don't respect .gitignore
            .arg("--hidden")         // Include hidden files
            .arg(keyword)            // Search pattern
            .arg(directory)          // Directory to search
            .output()
            .map_err(|e| format!("Failed to execute ripgrep: {}", e))?;

        if !output.status.success() && !output.stdout.is_empty() {
            // If ripgrep fails but has output, we still try to parse it
            tracing::warn!("ripgrep exited with non-zero status: {}", output.status);
        }

        // Parse the JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = String::new();
        let mut count = 0;

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse the JSON line
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    if let Some(match_type) = json.get("type").and_then(|t| t.as_str()) {
                        if match_type == "match" {
                            if let (Some(path), Some(lines)) = (
                                json.get("data").and_then(|d| d.get("path")).and_then(|p| p.get("text")).and_then(|t| t.as_str()),
                                json.get("data").and_then(|d| d.get("lines")).and_then(|l| l.get("text")).and_then(|t| t.as_str()),
                            ) {
                                count += 1;
                                results.push_str(&format!("Hit: {} - {}\n", path, lines.trim()));
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse ripgrep JSON output: {}", e);
                    tracing::debug!("Problematic line: {}", line);
                }
            }
        }

        if count == 0 {
            // Fallback to grep-rs library if ripgrep command fails or returns no results
            return self.search_with_grep_rs(directory, keyword);
        }

        Ok(results)
    }

    // Fallback method using the grep-rs library
    fn search_with_grep_rs(&self, directory: &str, pattern: &str) -> Result<String, String> {
        // Create a matcher for the search pattern
        let _matcher = RegexMatcher::new(pattern)
            .map_err(|e| format!("Invalid search pattern: {}", e))?;

        let mut results = String::new();
        let mut count = 0;

        // Configure the searcher for potential future use
        let _searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(true)
            .build();

        // Walk through files in the directory
        let walker = WalkBuilder::new(directory)
            .hidden(false)        // Include hidden files
            .ignore(false)        // Don't respect .gitignore
            .max_depth(Some(10))  // Limit directory depth
            .build();

        for result in walker {
            if count >= 10 {
                // Limit to 10 results for performance
                break;
            }

            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        // Try to search the file
                        match fs::read_to_string(path) {
                            Ok(content) => {
                                if content.contains(pattern) {
                                    count += 1;
                                    // Find the matching line
                                    for (i, line) in content.lines().enumerate() {
                                        if line.contains(pattern) {
                                            results.push_str(&format!("Hit: {} - Line {}: {}\n", 
                                                path.display(), i + 1, line.trim()));
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!("Could not read file {}: {}", path.display(), e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!("Error walking directory: {}", e);
                }
            }
        }

        Ok(results)
    }
}

#[tool(tool_box)]
impl ServerHandler for SearchTool {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides file search capabilities using ripgrep for fast and efficient searching."
                    .into(),
            ),
        }
    }
}
