use super::GitHubClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowRun {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowRunsResponse {
    pub total_count: i32,
    pub workflow_runs: Vec<WorkflowRun>,
}

impl GitHubClient {
    pub async fn get_latest_workflow_run(&self) -> Result<Option<WorkflowRun>> {
        let path = "actions/runs?per_page=1";
        let response: WorkflowRunsResponse = self.get_rest(path).await?;
        Ok(response.workflow_runs.into_iter().next())
    }
}
