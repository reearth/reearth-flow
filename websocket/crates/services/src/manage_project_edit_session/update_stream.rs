use chrono::Utc;
use tracing::debug;

impl<R, S, M> ManageEditSessionService<R, S, M>
where
    R: ProjectEditingSessionRepository<Error = ProjectRepositoryError> + Send + Sync + 'static,
    S: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync + 'static,
    M: RedisDataManager<Error = FlowProjectRedisDataManagerError> + Send + Sync + 'static,
{
    async fn update_stream(
        &self,
        project_id: &str,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        // 检查并合并更新
        if let Ok((merged_update, updated_by)) = self.redis_data_manager.merge_updates(false).await
        {
            // 更新最后合并时间
            let mut last_merged_at = data.last_merged_at.write().await;
            *last_merged_at = Some(Utc::now());
            debug!("Updates merged for project: {}", project_id);

            // 如果有更新，推送到 Redis
            if !merged_update.is_empty() {
                self.redis_data_manager
                    .push_update(merged_update, Some("system".to_string()))
                    .await?;
                debug!("Updates pushed to Redis for project: {}", project_id);
            }
        }

        Ok(())
    }
}
