use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionBlock {
    pub id: String,
    pub r#type: String,
    #[serde(flatten)]
    pub content: serde_json::Value,
}

pub fn parse_blocks_to_markdown(blocks: Vec<NotionBlock>) -> String {
    let mut markdown = String::new();
    for block in blocks {
        let block_md = match block.r#type.as_str() {
            "paragraph" => parse_rich_text(&block.content["paragraph"]["rich_text"]),
            "heading_1" => format!(
                "# {}\n",
                parse_rich_text(&block.content["heading_1"]["rich_text"])
            ),
            "heading_2" => format!(
                "## {}\n",
                parse_rich_text(&block.content["heading_2"]["rich_text"])
            ),
            "heading_3" => format!(
                "### {}\n",
                parse_rich_text(&block.content["heading_3"]["rich_text"])
            ),
            "bulleted_list_item" => format!(
                "- {}\n",
                parse_rich_text(&block.content["bulleted_list_item"]["rich_text"])
            ),
            "numbered_list_item" => format!(
                "1. {}\n",
                parse_rich_text(&block.content["numbered_list_item"]["rich_text"])
            ),
            "to_do" => {
                let checked = block.content["to_do"]["checked"].as_bool().unwrap_or(false);
                format!(
                    "- [{}] {}\n",
                    if checked { "x" } else { " " },
                    parse_rich_text(&block.content["to_do"]["rich_text"])
                )
            }
            "code" => {
                let lang = block.content["code"]["language"].as_str().unwrap_or("text");
                format!(
                    "```{}\n{}\n```\n",
                    lang,
                    parse_rich_text(&block.content["code"]["rich_text"])
                )
            }
            "quote" => format!(
                "> {}\n",
                parse_rich_text(&block.content["quote"]["rich_text"])
            ),
            "divider" => "---\n".to_string(),
            _ => String::new(), // Skip unsupported types for now
        };
        markdown.push_str(&block_md);
        if !block_md.is_empty() && !block_md.ends_with('\n') {
            markdown.push('\n');
        }
    }
    markdown
}

fn parse_rich_text(rich_text: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(arr) = rich_text.as_array() {
        for item in arr {
            if let Some(plain_text) = item["plain_text"].as_str() {
                text.push_str(plain_text);
            }
        }
    }
    text
}
