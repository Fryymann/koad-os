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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_block(block_type: &str, inner: serde_json::Value) -> NotionBlock {
        NotionBlock {
            id: "test-id".to_string(),
            r#type: block_type.to_string(),
            content: json!({ block_type: inner }),
        }
    }

    fn rich_text_val(text: &str) -> serde_json::Value {
        json!([{"plain_text": text, "type": "text"}])
    }

    #[test]
    fn empty_blocks_produces_empty_string() {
        assert_eq!(parse_blocks_to_markdown(vec![]), "");
    }

    #[test]
    fn paragraph_produces_plain_text_with_newline() {
        let block = make_block(
            "paragraph",
            json!({"rich_text": rich_text_val("Hello world")}),
        );
        assert_eq!(parse_blocks_to_markdown(vec![block]), "Hello world\n");
    }

    #[test]
    fn heading_1_adds_single_hash() {
        let block = make_block("heading_1", json!({"rich_text": rich_text_val("Title")}));
        assert_eq!(parse_blocks_to_markdown(vec![block]), "# Title\n");
    }

    #[test]
    fn heading_2_adds_double_hash() {
        let block = make_block("heading_2", json!({"rich_text": rich_text_val("Section")}));
        assert_eq!(parse_blocks_to_markdown(vec![block]), "## Section\n");
    }

    #[test]
    fn heading_3_adds_triple_hash() {
        let block = make_block(
            "heading_3",
            json!({"rich_text": rich_text_val("Subsection")}),
        );
        assert_eq!(parse_blocks_to_markdown(vec![block]), "### Subsection\n");
    }

    #[test]
    fn bulleted_list_item_adds_dash_prefix() {
        let block = make_block(
            "bulleted_list_item",
            json!({"rich_text": rich_text_val("Item")}),
        );
        assert_eq!(parse_blocks_to_markdown(vec![block]), "- Item\n");
    }

    #[test]
    fn numbered_list_item_adds_number_prefix() {
        let block = make_block(
            "numbered_list_item",
            json!({"rich_text": rich_text_val("First")}),
        );
        assert_eq!(parse_blocks_to_markdown(vec![block]), "1. First\n");
    }

    #[test]
    fn todo_unchecked_renders_empty_checkbox() {
        let block = make_block(
            "to_do",
            json!({"rich_text": rich_text_val("Task"), "checked": false}),
        );
        assert_eq!(parse_blocks_to_markdown(vec![block]), "- [ ] Task\n");
    }

    #[test]
    fn todo_checked_renders_x_checkbox() {
        let block = make_block(
            "to_do",
            json!({"rich_text": rich_text_val("Done"), "checked": true}),
        );
        assert_eq!(parse_blocks_to_markdown(vec![block]), "- [x] Done\n");
    }

    #[test]
    fn todo_defaults_to_unchecked_when_field_absent() {
        let block = make_block("to_do", json!({"rich_text": rich_text_val("Pending")}));
        assert_eq!(parse_blocks_to_markdown(vec![block]), "- [ ] Pending\n");
    }

    #[test]
    fn code_block_wraps_in_fences_with_language() {
        let block = make_block(
            "code",
            json!({"rich_text": rich_text_val("let x = 1;"), "language": "rust"}),
        );
        assert_eq!(
            parse_blocks_to_markdown(vec![block]),
            "```rust\nlet x = 1;\n```\n"
        );
    }

    #[test]
    fn code_block_defaults_to_text_language() {
        let block = make_block("code", json!({"rich_text": rich_text_val("some code")}));
        assert_eq!(
            parse_blocks_to_markdown(vec![block]),
            "```text\nsome code\n```\n"
        );
    }

    #[test]
    fn quote_adds_blockquote_prefix() {
        let block = make_block("quote", json!({"rich_text": rich_text_val("Famous words")}));
        assert_eq!(parse_blocks_to_markdown(vec![block]), "> Famous words\n");
    }

    #[test]
    fn divider_produces_horizontal_rule() {
        let block = NotionBlock {
            id: "div-id".to_string(),
            r#type: "divider".to_string(),
            content: json!({"divider": {}}),
        };
        assert_eq!(parse_blocks_to_markdown(vec![block]), "---\n");
    }

    #[test]
    fn unknown_block_type_is_skipped() {
        let block = NotionBlock {
            id: "unk-id".to_string(),
            r#type: "unsupported_type".to_string(),
            content: json!({}),
        };
        assert_eq!(parse_blocks_to_markdown(vec![block]), "");
    }

    #[test]
    fn multiple_rich_text_spans_are_concatenated() {
        let block = NotionBlock {
            id: "multi-id".to_string(),
            r#type: "paragraph".to_string(),
            content: json!({
                "paragraph": {
                    "rich_text": [
                        {"plain_text": "Hello", "type": "text"},
                        {"plain_text": " world", "type": "text"}
                    ]
                }
            }),
        };
        assert_eq!(parse_blocks_to_markdown(vec![block]), "Hello world\n");
    }

    #[test]
    fn multiple_blocks_are_concatenated_in_order() {
        let h1 = make_block("heading_1", json!({"rich_text": rich_text_val("Title")}));
        let para = make_block("paragraph", json!({"rich_text": rich_text_val("Body")}));
        let divider = NotionBlock {
            id: "div".to_string(),
            r#type: "divider".to_string(),
            content: json!({"divider": {}}),
        };
        assert_eq!(
            parse_blocks_to_markdown(vec![h1, para, divider]),
            "# Title\nBody\n---\n"
        );
    }
}
