# KoadOS — The Koados Citadel

**KoadOS** is a **Citadel-class Station** and the mothership of the Koad agent fleet. It is a programmatic-first framework for building a long-term, context-aware partnership between an **Admiral (Human)** and a **Captain (AI Agent)**.

The Citadel serves as the central intelligence hub, coordinating forward-deployed stations (like the **SLE — Skylinks Local Ecosystem**) and managing the orchestration, memory, and runtime protocols for the entire crew.

## 🚀 Key Features

- **Citadel Core**: High-performance Rust-based gRPC backbone (kspine).
- **Command Deck**: A powerful Rust CLI (koad) for station management.
- **Engine Room**: Ultra-fast Redis-backed state management and PubSub.
- **Crew Hierarchy**: Multi-tier agent ranking (Captain, Chief Officer, Engineer).
- **Integrated Bridges**: Native connectors for Notion, GCP, Airtable, and Stripe.

## 🛠️ Getting Started

To install KoadOS and initialize your first partner, follow our **[Onboarding Guide](docs/ONBOARDING.md)**.

```bash
git clone https://github.com/Fryymann/koad-os.git ~/.koad-os
cd ~/.koad-os
./koad-setup.sh --partner "YourName" --persona "Koad" --role "Captain" --langs "Rust,Python,Node.js"
```

## 📖 Documentation

- **[Crew Hierarchy](docs/protocols/CREW_HIERARCHY.md)**: Ranks, Roles, and the SLE/SCE station definitions.
- **[Onboarding](docs/ONBOARDING.md)**: Your first 15 minutes.
- **[Developer Guide](docs/DEVELOPMENT.md)**: How to write skills and contribute to the core.
- **[Architecture](docs/ARCHITECTURE.md)**: Deep dive into the Spine, Memory, and Drivers.
- **[Technical Spec](SPEC.md)**: Full CLI reference and database schema.

## 🤝 Contributing

We welcome contributions! Whether you're adding a new global skill or improving the Rust core, please check our **[Contributor Manifesto](docs/protocols/CONTRIBUTOR_MANIFESTO.md)** to get started.

## ⚖️ License

MIT License. See [LICENSE.md](LICENSE.md) for details. (Drafting soon!)
