use crate::parser::{parse_blocks_to_markdown, NotionBlock};
use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use serde_json::Value;

pub struct NotionClient {
    client: Client,
}

impl NotionClient {
    pub fn new(api_key: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        headers.insert(
            "Notion-Version",
            header::HeaderValue::from_static("2022-06-28"),
        );
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client })
    }

    pub async fn get_page_content_markdown(&self, block_id: &str) -> Result<String> {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", block_id);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await?;
            return Err(anyhow!("Notion API Error: {}", err_text));
        }

        let body: Value = response.json().await?;
        let blocks: Vec<NotionBlock> = serde_json::from_value(body["results"].clone())?;

        Ok(parse_blocks_to_markdown(blocks))
    }

    pub async fn query_database(&self, database_id: &str) -> Result<Vec<(String, String, Value)>> {
        let url = format!("https://api.notion.com/v1/databases/{}/query", database_id);
        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await?;
            return Err(anyhow!("Notion API Error: {}", err_text));
        }

        let body: Value = response.json().await?;
        let results = body["results"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid response from Notion API: results is not an array"))?;

        let mut pages = Vec::new();
        for page in results {
            let id = page["id"]
                .as_str()
                .ok_or_else(|| anyhow!("Page ID missing"))?
                .to_string();

            let mut title = "Untitled".to_string();
            let properties = page["properties"].clone();

            if let Some(props_obj) = properties.as_object() {
                for (_, prop) in props_obj {
                    if prop["type"] == "title" {
                        if let Some(title_arr) = prop["title"].as_array() {
                            let mut title_text = String::new();
                            for item in title_arr {
                                if let Some(plain_text) = item["plain_text"].as_str() {
                                    title_text.push_str(plain_text);
                                }
                            }
                            if !title_text.is_empty() {
                                title = title_text;
                                break;
                            }
                        }
                    }
                }
            }
            pages.push((id, title, properties));
        }

        Ok(pages)
    }

    pub async fn update_page_property(
        &self,
        page_id: &str,
        property_name: &str,
        value: Value,
    ) -> Result<()> {
        let url = format!("https://api.notion.com/v1/pages/{}", page_id);
        let body = serde_json::json!({
            "properties": {
                property_name: value
            }
        });

        let response = self.client.patch(&url).json(&body).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await?;
            return Err(anyhow!("Notion API Error: {}", err_text));
        }

        Ok(())
    }

    pub async fn post_to_stream(
        &self,
        database_id: &str,
        author: &str,
        target: &str,
        topic: &str,
        priority: &str,
    ) -> Result<()> {
        let url = "https://api.notion.com/v1/pages";
        let body = serde_json::json!({
            "parent": { "database_id": database_id },
            "properties": {
                "Topic": { "title": [{ "text": { "content": topic } }] },
                "Author": { "select": { "name": author } },
                "Target": { "select": { "name": target } },
                "Priority": { "select": { "name": priority } },
                "Type": { "select": { "name": "Update" } },
                "Status": { "select": { "name": "Unread" } }
            }
        });

        let response = self.client.post(url).json(&body).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await?;
            return Err(anyhow!("Notion API Error: {}", err_text));
        }

        Ok(())
    }
}
