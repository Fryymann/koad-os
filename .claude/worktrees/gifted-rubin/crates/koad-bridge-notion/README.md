# koad-bridge-notion

The optimized native Notion bridge for **KoadOS**.

## 🏗 Overview

`koad-bridge-notion` provides the high-performance Rust client for the Notion API, specifically optimized for the **Notion Context Sink** and **KoadStream** services.

## 🛠 Features

- **Page Parser**: Surgically parse Notion pages into KoadOS-compatible Markdown.
- **Stream Integration**: Post high-priority events and agent signals to the Notion-based KoadStream.
- **Durable Sync**: Interface for bidirectional synchronization between local memory and Notion databases.

## 🔑 Key Modules

- **`client`**: Notion REST API client using `reqwest`.
- **`parser`**: Block-to-Markdown conversion logic.
- **`stream`**: Specialized handling for the KoadStream event protocol.

## 🚀 Usage

```toml
[dependencies]
koad-bridge-notion = { path = "../koad-bridge-notion" }
```
