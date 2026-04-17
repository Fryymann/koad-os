# koad-board

The GitHub Project Board and Command Deck integration layer for **KoadOS**.

## 🏗 Overview

`koad-board` is the primary bridge between the KoadOS memory bank and the external GitHub Project V2 boards. It enables the **Sovereign GitHub Protocol (SGP)** for task tracking, issue spawning, and state synchronization.

## 🛠 Features

- **GitHub Client**: A high-level wrapper around the GitHub REST and GraphQL APIs.
- **Project V2 Support**: Native support for the modern GitHub Projects API (Board #2).
- **Issue Lifecycle**: Automated issue creation, status transitions, and metadata synchronization.
- **Path-Aware Auth**: Integration with the directory-specific GITHUB PAT selection logic.

## 🔑 Key Modules

- **`client`**: Core GitHub API logic using `reqwest`.
- **`issue`**: Issue domain models and status enums (Todo, InProgress, Done).
- **`project`**: Logic for interacting with Project V2 nodes and fields.

## 🚀 Usage

This crate is primarily used as a dependency of `koad-cli` and `koad-citadel`.

```toml
[dependencies]
koad-board = { path = "../koad-board" }
```
