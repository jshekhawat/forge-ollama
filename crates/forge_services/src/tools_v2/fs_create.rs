use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use bytes::Bytes;
use forge_app::{FsCreateOutput, FsCreateService};

use crate::utils::assert_absolute_path;
use crate::{FsCreateDirsService, FsMetaService, FsReadService, FsWriteService, Infrastructure};

/// Use it to create a new file at a specified path with the provided content.
/// Always provide absolute paths for file locations. The tool
/// automatically handles the creation of any missing intermediary directories
/// in the specified path.
/// IMPORTANT: DO NOT attempt to use this tool to move or rename files, use the
/// shell tool instead.
pub struct ForgeFsCreate<F>(Arc<F>);

impl<F: Infrastructure> ForgeFsCreate<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self(infra)
    }
}

#[async_trait::async_trait]
impl<F: Infrastructure> FsCreateService for ForgeFsCreate<F> {
    async fn create(
        &self,
        path: String,
        content: String,
        overwrite: bool,
        capture_snapshot: bool,
    ) -> anyhow::Result<FsCreateOutput> {
        let path = Path::new(&path);
        assert_absolute_path(path)?;
        // Validate file content if it's a supported language file
        let syntax_warning = super::syn::validate(path, &content);
        if let Some(parent) = Path::new(&path).parent() {
            self.0
                .create_dirs_service()
                .create_dirs(parent)
                .await
                .with_context(|| format!("Failed to create directories: {}", path.display()))?;
        }
        // Check if the file exists
        let file_exists = self.0.file_meta_service().is_file(path).await?;

        // If file exists and overwrite flag is not set, return an error with the
        // existing content
        if file_exists && !overwrite {
            // Special message for the LLM
            return Err(anyhow::anyhow!(
                "Cannot overwrite existing file: overwrite flag not set.",
            ))
            // What the user sees
            .with_context(|| format!("File already exists at {}", path.display()));
        }

        // record the file content before they're modified
        let old_content = if file_exists && overwrite {
            Some(self.0.file_read_service().read_utf8(path).await?)
        } else {
            None
        };

        // Write file only after validation passes and directories are created
        self.0
            .file_write_service()
            .write(path, Bytes::from(content), capture_snapshot)
            .await?;

        Ok(FsCreateOutput {
            path: path.display().to_string(),
            previous: old_content,
            warning: syntax_warning.map(|v| v.to_string()),
        })
    }
}
