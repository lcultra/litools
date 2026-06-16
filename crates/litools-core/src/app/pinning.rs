use litools_index::repository::PinnedRepository;

use crate::{
    app::LitoolsApp,
    error::{LitoolsError, LitoolsResult},
    pinning_service,
};

impl LitoolsApp {
    pub fn pin_result(&self, result_id: impl Into<String>) -> LitoolsResult<()> {
        let result_id = result_id.into();
        let (target_type, target_id) =
            pinning_service::validate_target(&self.context.database, &result_id)?;
        pinning_service::pin(&self.context.database, target_type, &target_id)
    }

    pub fn unpin_result(&self, result_id: impl Into<String>) -> LitoolsResult<()> {
        let result_id = result_id.into();
        let (target_type, target_id) =
            pinning_service::validate_target(&self.context.database, &result_id)?;
        pinning_service::unpin(&self.context.database, target_type, &target_id)
    }

    pub fn reorder_pinned_results(&self, result_ids: Vec<String>) -> LitoolsResult<()> {
        let mut targets = Vec::with_capacity(result_ids.len());

        for result_id in &result_ids {
            let (target_type, target_id) =
                pinning_service::validate_target(&self.context.database, result_id)?;
            targets.push((target_type.to_string(), target_id, result_id.clone()));
        }

        // 验证所有目标已固定
        self.context.database.read(|conn| {
            let pinned = PinnedRepository::new(&conn);
            for (target_type, target_id, result_id) in &targets {
                if !pinned.is_pinned(target_type, target_id)? {
                    return Err(LitoolsError::CommandNotFound(result_id.clone()));
                }
            }
            Ok(())
        })?;

        let ordered: Vec<(String, String)> = targets
            .into_iter()
            .map(|(tt, tid, _)| (tt, tid))
            .collect();
        pinning_service::reorder(&self.context.database, &ordered)
    }

    pub(crate) fn is_target_pinned(
        &self,
        target_type: &str,
        target_id: &str,
    ) -> LitoolsResult<bool> {
        pinning_service::is_pinned(&self.context.database, target_type, target_id)
    }

    pub(crate) fn target_from_result_id(&self, result_id: &str) -> Option<(&'static str, String)> {
        pinning_service::parse_target(result_id)
    }
}
