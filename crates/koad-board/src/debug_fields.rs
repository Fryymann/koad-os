use anyhow::Result;
use koad_board::GitHubClient;
use koad_core::config::KoadConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let config = KoadConfig::load()?;
    let token = config.resolve_gh_token()?;
    let owner = config.get_github_owner()?;
    let repo = config.get_github_repo()?;

    let client = GitHubClient::new(token, owner, repo)?;
    let project_id = client.get_project_id(2).await?;

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

    let data: serde_json::Value = client.graphql(query, variables).await?;
    println!("Project Fields Metadata:
{}", serde_json::to_string_pretty(&data)?);

    Ok(())
}
