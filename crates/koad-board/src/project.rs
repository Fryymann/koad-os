use super::GitHubClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub target_version: Option<String>,
    pub number: Option<i32>,
}

impl GitHubClient {
    pub async fn get_project_id(&self, number: i32) -> Result<String> {
        let query = r#"
            query($owner: String!, $number: Int!) {
              user(login: $owner) {
                projectV2(number: $number) {
                  id
                }
              }
            }
        "#;

        let variables = serde_json::json!({
            "owner": self.owner,
            "number": number,
        });

        let data: serde_json::Value = self.graphql(query, variables).await?;
        data["user"]["projectV2"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Project not found"))
    }

    pub async fn update_item_field(&self, project_id: &str, item_id: &str, field_id: &str, value: serde_json::Value) -> Result<()> {
        let query = r#"
            mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $value: ProjectV2FieldValue!) {
              updateProjectV2ItemFieldValue(input: {
                projectId: $projectId,
                itemId: $itemId,
                fieldId: $fieldId,
                value: $value
              }) {
                projectV2Item {
                  id
                }
              }
            }
        "#;

        let variables = serde_json::json!({
            "projectId": project_id,
            "itemId": item_id,
            "fieldId": field_id,
            "value": value,
        });

        let _: serde_json::Value = self.graphql(query, variables).await?;
        Ok(())
    }

    pub async fn get_status_field_id(&self, project_id: &str) -> Result<String> {
        let query = r#"
            query($projectId: ID!) {
              node(id: $projectId) {
                ... on ProjectV2 {
                  fields(first: 50) {
                    nodes {
                      ... on ProjectV2Field {
                        id
                        name
                      }
                      ... on ProjectV2SingleSelectField {
                        id
                        name
                      }
                    }
                  }
                }
              }
            }
        "#;

        let variables = serde_json::json!({
            "projectId": project_id,
        });

        let data: serde_json::Value = self.graphql(query, variables).await?;
        let fields = data["node"]["fields"]["nodes"].as_array().ok_or_else(|| anyhow::anyhow!("No fields found"))?;
        
        for field in fields {
            if field["name"].as_str() == Some("Status") {
                return field["id"].as_str().map(|s| s.to_string()).ok_or_else(|| anyhow::anyhow!("Field ID missing"));
            }
        }
        
        anyhow::bail!("Status field not found in project")
    }

    pub async fn get_status_option_id(&self, project_id: &str, option_name: &str) -> Result<String> {
        let query = r#"
            query($projectId: ID!) {
              node(id: $projectId) {
                ... on ProjectV2 {
                  fields(first: 50) {
                    nodes {
                      ... on ProjectV2SingleSelectField {
                        name
                        options {
                          id
                          name
                        }
                      }
                    }
                  }
                }
              }
            }
        "#;

        let variables = serde_json::json!({
            "projectId": project_id,
        });

        let data: serde_json::Value = self.graphql(query, variables).await?;
        let fields = data["node"]["fields"]["nodes"].as_array().ok_or_else(|| anyhow::anyhow!("No fields found"))?;
        
        for field in fields {
            if field["name"].as_str() == Some("Status") {
                let options = field["options"].as_array().ok_or_else(|| anyhow::anyhow!("No options found for Status field"))?;
                for option in options {
                    if option["name"].as_str() == Some(option_name) {
                        return option["id"].as_str().map(|s| s.to_string()).ok_or_else(|| anyhow::anyhow!("Option ID missing"));
                    }
                }
            }
        }
        
        anyhow::bail!("Status option '{}' not found in project", option_name)
    }

    pub async fn list_project_items(&self, project_number: i32) -> Result<Vec<ProjectItem>> {
        let query = r#"
            query($owner: String!, $number: Int!) {
              user(login: $owner) {
                projectV2(number: $number) {
                  items(first: 100) {
                    nodes {
                      id
                      content {
                        ... on Issue {
                          title
                          number
                        }
                      }
                      fieldValues(first: 20) {
                        nodes {
                          ... on ProjectV2ItemFieldSingleSelectValue {
                            name
                            field {
                              ... on ProjectV2FieldCommon {
                                name
                              }
                            }
                          }
                          ... on ProjectV2ItemFieldDateValue {
                            date
                            field {
                              ... on ProjectV2FieldCommon {
                                name
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
        "#;

        let variables = serde_json::json!({
            "owner": self.owner,
            "number": project_number,
        });

        let data: serde_json::Value = self.graphql(query, variables).await?;
        
        let mut items = Vec::new();
        if let Some(nodes) = data.get("user").and_then(|u| u.get("projectV2")).and_then(|p| p.get("items")).and_then(|i| i.get("nodes")) {
            for node in nodes.as_array().unwrap_or(&vec![]) {
                let id = node["id"].as_str().unwrap_or_default().to_string();
                let title = node["content"]["title"].as_str().unwrap_or_default().to_string();
                let number = node["content"]["number"].as_i64().map(|n| n as i32);
                
                let mut status = "Unknown".to_string();
                let mut start_date = None;
                let mut target_date = None;
                let mut target_version = None;

                if let Some(values) = node["fieldValues"]["nodes"].as_array() {
                    for val in values {
                        let field_name = val["field"]["name"].as_str().unwrap_or_default();
                        match field_name {
                            "Status" => {
                                status = val["name"].as_str().unwrap_or("Unknown").to_string();
                            }
                            "Start Date" => {
                                start_date = val["date"].as_str().map(|s| s.to_string());
                            }
                            "Target Date" => {
                                target_date = val["date"].as_str().map(|s| s.to_string());
                            }
                            "Target Version" => {
                                target_version = val["name"].as_str().map(|s| s.to_string());
                            }
                            _ => {}
                        }
                    }
                }

                items.push(ProjectItem {
                    id,
                    title,
                    status,
                    start_date,
                    target_date,
                    target_version,
                    number,
                });
            }
        }

        Ok(items)
    }

    pub async fn add_item_to_project(&self, project_id: &str, content_id: &str) -> Result<String> {
        let query = r#"
            mutation($projectId: ID!, $contentId: ID!) {
              addProjectV2ItemById(input: {
                projectId: $projectId,
                contentId: $contentId
              }) {
                item {
                  id
                }
              }
            }
        "#;

        let variables = serde_json::json!({
            "projectId": project_id,
            "contentId": content_id,
        });

        let data: serde_json::Value = self.graphql(query, variables).await?;
        data["addProjectV2ItemById"]["item"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Failed to add item to project"))
    }

    pub async fn get_repository_id(&self) -> Result<String> {
        let query = r#"
            query($owner: String!, $repo: String!) {
              repository(owner: $owner, name: $repo) {
                id
              }
            }
        "#;

        let variables = serde_json::json!({
            "owner": self.owner,
            "repo": self.repo,
        });

        let data: serde_json::Value = self.graphql(query, variables).await?;
        data["repository"]["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Repository not found"))
    }

    pub async fn list_open_issues(&self) -> Result<Vec<(String, i32, String)>> {
        let query = r#"
            query($owner: String!, $repo: String!) {
              repository(owner: $owner, name: $repo) {
                issues(first: 100, states: OPEN) {
                  nodes {
                    id
                    number
                    title
                  }
                }
              }
            }
        "#;

        let variables = serde_json::json!({
            "owner": self.owner,
            "repo": self.repo,
        });

        let data: serde_json::Value = self.graphql(query, variables).await?;
        
        let mut issues = Vec::new();
        if let Some(nodes) = data.get("repository").and_then(|r| r.get("issues")).and_then(|i| i.get("nodes")) {
            for node in nodes.as_array().unwrap_or(&vec![]) {
                let id = node["id"].as_str().unwrap_or_default().to_string();
                let number = node["number"].as_i64().unwrap_or_default() as i32;
                let title = node["title"].as_str().unwrap_or_default().to_string();
                issues.push((id, number, title));
            }
        }
        Ok(issues)
    }
}
