use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use csv::Writer;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRole {
    All,
    Developer,
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::All => write!(f, "all"),
            UserRole::Developer => write!(f, "developer"),
            UserRole::Admin => write!(f, "admin"),
        }
    }
}

/// Base trait for documentation items
trait DocItem: Serialize {
    fn role(&self) -> UserRole;
}

/// User guide documentation
#[derive(Debug, Serialize, Deserialize)]
pub struct UserGuideDoc {
    id: String,
    title: String,
    content: String,
    category: String,
    role: String,
    order: i32,
}

impl DocItem for UserGuideDoc {
    fn role(&self) -> UserRole {
        UserRole::All
    }
}

/// Setup guide documentation
#[derive(Debug, Serialize, Deserialize)]
pub struct SetupGuideDoc {
    id: String,
    title: String,
    steps: String,
    prerequisites: String,
    category: String,
    role: String,
    order: i32,
}

impl DocItem for SetupGuideDoc {
    fn role(&self) -> UserRole {
        UserRole::All
    }
}

/// CLI command documentation
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandDoc {
    id: String,
    name: String,
    description: String,
    usage_example: String,
    category: String,
    role: String,
    is_required: bool,
}

impl DocItem for CommandDoc {
    fn role(&self) -> UserRole {
        if self.category == "Advanced" {
            UserRole::Developer
        } else {
            UserRole::All
        }
    }
}

/// Command parameter documentation
#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterDoc {
    id: String,
    command_id: String,
    name: String,
    description: String,
    data_type: String,
    default_value: Option<String>,
    role: String,
    is_required: bool,
}

impl DocItem for ParameterDoc {
    fn role(&self) -> UserRole {
        UserRole::All
    }
}

/// Configuration documentation
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDoc {
    id: String,
    name: String,
    description: String,
    data_type: String,
    default_value: Option<String>,
    category: String,
    role: String,
}

impl DocItem for ConfigDoc {
    fn role(&self) -> UserRole {
        match self.category.as_str() {
            "Advanced" | "Development" => UserRole::Developer,
            _ => UserRole::All,
        }
    }
}

/// Technical documentation for developers
#[derive(Debug, Serialize, Deserialize)]
pub struct TechnicalDoc {
    id: String,
    title: String,
    content: String,
    category: String,
    role: String,
    related_files: String,
}

impl DocItem for TechnicalDoc {
    fn role(&self) -> UserRole {
        UserRole::Developer
    }
}

/// Report template documentation
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportDoc {
    id: String,
    name: String,
    description: String,
    format: String,
    fields: String,
    role: String,
    category: String,
}

impl DocItem for ReportDoc {
    fn role(&self) -> UserRole {
        UserRole::All
    }
}

/// Troubleshooting guide
#[derive(Debug, Serialize, Deserialize)]
pub struct TroubleshootingDoc {
    id: String,
    issue: String,
    solution: String,
    category: String,
    role: String,
    related_errors: String,
}

impl DocItem for TroubleshootingDoc {
    fn role(&self) -> UserRole {
        UserRole::All
    }
}

/// Role permissions
#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermission {
    role: String,
    resource: String,
    permissions: String,
    description: String,
}

/// Generates documentation in CSV format
pub struct DocGenerator {
    output_dir: PathBuf,
}

impl DocGenerator {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Generates all documentation tables
    pub async fn generate_docs(&self) -> Result<()> {
        fs::create_dir_all(&self.output_dir).await?;
        
        // Generate user documentation
        self.generate_user_guides().await?;
        self.generate_setup_guides().await?;
        self.generate_reports().await?;
        self.generate_troubleshooting().await?;
        
        // Generate developer documentation
        self.generate_technical_docs().await?;
        self.generate_commands().await?;
        self.generate_parameters().await?;
        self.generate_configs().await?;
        
        // Generate role permissions
        self.generate_roles().await?;
        
        Ok(())
    }

    async fn generate_user_guides(&self) -> Result<()> {
        let guides = vec![
            UserGuideDoc {
                id: "ug_basic".to_string(),
                title: "Basic Usage".to_string(),
                content: "Learn how to use basic CLI commands...".to_string(),
                category: "Getting Started".to_string(),
                role: "all".to_string(),
                order: 1,
            },
            UserGuideDoc {
                id: "ug_sync".to_string(),
                title: "File Synchronization".to_string(),
                content: "How to sync files between systems...".to_string(),
                category: "Core Features".to_string(),
                role: "all".to_string(),
                order: 2,
            },
        ];

        self.write_csv("user_guides.csv", &guides).await
    }

    async fn generate_setup_guides(&self) -> Result<()> {
        let guides = vec![
            SetupGuideDoc {
                id: "setup_basic".to_string(),
                title: "Basic Setup".to_string(),
                steps: "1. Install CLI\n2. Configure paths...".to_string(),
                prerequisites: "Rust installed".to_string(),
                category: "Installation".to_string(),
                role: "all".to_string(),
                order: 1,
            },
        ];

        self.write_csv("setup_guides.csv", &guides).await
    }

    async fn generate_technical_docs(&self) -> Result<()> {
        let docs = vec![
            TechnicalDoc {
                id: "tech_arch".to_string(),
                title: "Architecture Overview".to_string(),
                content: "System architecture and components...".to_string(),
                category: "Architecture".to_string(),
                role: "developer".to_string(),
                related_files: "main.rs,sync.rs".to_string(),
            },
        ];

        self.write_csv("technical_docs.csv", &docs).await
    }

    async fn generate_reports(&self) -> Result<()> {
        let reports = vec![
            ReportDoc {
                id: "report_sync".to_string(),
                name: "Sync Report".to_string(),
                description: "Details of file synchronization".to_string(),
                format: "CSV".to_string(),
                fields: "filename,status,timestamp".to_string(),
                role: "all".to_string(),
                category: "Operations".to_string(),
            },
        ];

        self.write_csv("reports.csv", &reports).await
    }

    async fn generate_troubleshooting(&self) -> Result<()> {
        let guides = vec![
            TroubleshootingDoc {
                id: "trouble_conn".to_string(),
                issue: "Connection Failed".to_string(),
                solution: "Check network settings...".to_string(),
                category: "Network".to_string(),
                role: "all".to_string(),
                related_errors: "E001,E002".to_string(),
            },
        ];

        self.write_csv("troubleshooting.csv", &guides).await
    }

    async fn generate_roles(&self) -> Result<()> {
        let permissions = vec![
            RolePermission {
                role: "all".to_string(),
                resource: "user_guides".to_string(),
                permissions: "read".to_string(),
                description: "Access to user documentation".to_string(),
            },
            RolePermission {
                role: "developer".to_string(),
                resource: "technical_docs".to_string(),
                permissions: "read".to_string(),
                description: "Access to technical documentation".to_string(),
            },
        ];

        self.write_csv("role_permissions.csv", &permissions).await
    }

    async fn generate_commands(&self) -> Result<()> {
        let commands = vec![
            CommandDoc {
                id: "cmd_sync".to_string(),
                name: "sync".to_string(),
                description: "Synchronize files based on JSON configuration".to_string(),
                usage_example: "cli-file-sync sync --assets-source <URL>".to_string(),
                category: "Core".to_string(),
                role: "all".to_string(),
                is_required: false,
            },
            CommandDoc {
                id: "cmd_config".to_string(),
                name: "config".to_string(),
                description: "Configure CLI settings".to_string(),
                usage_example: "cli-file-sync config --base-url <URL>".to_string(),
                category: "Configuration".to_string(),
                role: "all".to_string(),
                is_required: false,
            },
        ];

        self.write_csv("commands.csv", &commands).await
    }

    async fn generate_parameters(&self) -> Result<()> {
        let parameters = vec![
            ParameterDoc {
                id: "param_assets_source".to_string(),
                command_id: "cmd_sync".to_string(),
                name: "assets-source".to_string(),
                description: "Source URL or path for assets JSON".to_string(),
                data_type: "string".to_string(),
                default_value: None,
                role: "all".to_string(),
                is_required: true,
            },
            ParameterDoc {
                id: "param_max_concurrent".to_string(),
                command_id: "cmd_sync".to_string(),
                name: "max-concurrent".to_string(),
                description: "Maximum number of concurrent downloads".to_string(),
                data_type: "integer".to_string(),
                default_value: Some("5".to_string()),
                role: "all".to_string(),
                is_required: false,
            },
        ];

        self.write_csv("parameters.csv", &parameters).await
    }

    async fn generate_configs(&self) -> Result<()> {
        let configs = vec![
            ConfigDoc {
                id: "cfg_base_url".to_string(),
                name: "base_url".to_string(),
                description: "Base URL for resolving relative paths".to_string(),
                data_type: "string".to_string(),
                default_value: None,
                category: "Network".to_string(),
                role: "all".to_string(),
            },
            ConfigDoc {
                id: "cfg_max_concurrent".to_string(),
                name: "max_concurrent".to_string(),
                description: "Maximum concurrent downloads".to_string(),
                data_type: "integer".to_string(),
                default_value: Some("5".to_string()),
                category: "Download".to_string(),
                role: "all".to_string(),
            },
        ];

        self.write_csv("configs.csv", &configs).await
    }

    async fn write_csv<T: serde::Serialize>(
        &self,
        filename: &str,
        data: &[T],
    ) -> Result<()> {
        let path = self.output_dir.join(filename);
        let mut wtr = Writer::from_path(&path)?;
        
        for item in data {
            wtr.serialize(item)?;
        }
        wtr.flush()?;
        
        Ok(())
    }
}
