use std::path::Path;
use std::sync::Arc;

use forge_app::{FsUndoOutput, FsUndoService};

use crate::utils::assert_absolute_path;
use crate::{FsReadService, FsSnapshotService, Infrastructure};

/// Reverts the most recent file operation (create/modify/delete) on a specific
/// file. Use this tool when you need to recover from incorrect file changes or
/// if a revert is requested by the user.
#[derive(Default)]
pub struct ForgeFsUndo<F>(Arc<F>);

impl<F: Infrastructure> ForgeFsUndo<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self(infra)
    }
}

#[async_trait::async_trait]
impl<F: Infrastructure> FsUndoService for ForgeFsUndo<F> {
    async fn undo(&self, path: String) -> anyhow::Result<FsUndoOutput> {
        let path = Path::new(&path);
        assert_absolute_path(path)?;
        let before_undo = self.0.file_read_service().read_utf8(path).await?;
        self.0.file_snapshot_service().undo_snapshot(path).await?;
        let after_undo = self.0.file_read_service().read_utf8(path).await?;

        Ok(FsUndoOutput { before_undo, after_undo })
    }
}
