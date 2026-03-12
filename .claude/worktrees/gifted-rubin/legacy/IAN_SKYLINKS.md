# AI Agent Handoff: Ian Deans (Skylinks at Buchanan Fields)

## One-sentence summary
Ian Deans is the Operations & Systems Administrator at Skylinks at Buchanan Fields (Concord, CA), acting as an operator-engineer who owns both day-to-day operational continuity and the software/integration systems that power the business. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

## Role identity (how to think about Ian)
- **Operator-engineer:** equal focus on real-world staff workflows and technical execution (code + configs + automations + vendor systems). [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Systems lead / principal full-stack developer:** default owner for Skylinks “technical systems,” especially website + online systems support. [^https://github.com/Skylinks-Golf/skylinks-wordpress-site/blob/main/TEAM_ROSTER.md]

## Job facts
- **Title:** Operations & Systems Administrator [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Department:** Administration [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Reports to:** Owner / General Manager [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Employment:** Full-Time [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Primary responsibility tags (directory):** Admin, Clubs (Schools/Golf Clubs), Tech Support, Tee Sheet Admin [^https://www.notion.so/2a8fe8ecae8f8010974fdb38a360f93d]

## What Ian owns (responsibility map)

### 1) Administrative + membership + golf-club operations
- Administer and maintain workflows for **NCGA/USGA golf club registration, compliance, and member administration**. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Manage membership processes using platforms such as **MemberPlanet** (and associated operational procedures). [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Coordinate with schools, golf clubs, and community orgs when those partnerships touch membership/club operations. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

### 2) Tee sheet and booking operations
- Own the **tee sheet administration** and operational correctness of online booking workflows. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c][^https://www.notion.so/2a8fe8ecae8f8010974fdb38a360f93d]
- Coordinate and maintain third-party booking integrations where applicable (e.g., GolfNow-related operations). [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

### 3) Inventory + supply chain systems (ops support + process controls)
- Build and refine processes for inventory tracking, purchasing controls, and receiving/reconciliation support.
- Collaborate with Pro Shop leads to reduce operational friction and error rates via clear workflows and tooling. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

### 4) Technical systems development (core engineering surface area)

#### Backend + business logic
- Design, develop, and maintain custom **membership and payment systems**. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Build backend APIs and services (explicitly framed as Node.js/Express patterns) and deploy business logic via cloud functions. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

#### Integrations + data management
- Integrate Skylinks operational systems across POS, payments, and data stores.
- Maintain Airtable as an operational source of truth where used, and implement automation/webhooks and data pipelines. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Typical integration pattern: **Stripe ↔ Airtable ↔ WordPress ↔ GCP** with validation, logging, and reconciliation. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

#### Payments
- Implement secure payment processing (Stripe is explicitly part of the role). [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

#### WordPress + website
- Maintain **skylinksgolf.com** with content updates and feature development. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Develop WordPress custom forms/components and integrate them with backend systems. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

### 5) Infrastructure + technical support (hands-on)
- Manage deployments and infrastructure in **Google Cloud Platform** (GCP). [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Provide technical support for facility tech (POS hardware, printers, routers, payment terminals) and coordinate vendor repairs/replacements when needed. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Maintain access controls and security posture for systems in scope. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

## Collaboration + routing rules (who gets what)
- **General questions:** Pro Shop staff / main phone line.
- **Schools and golf clubs:** route to LC.
- **Events and SkyRoom inquiries:** route to Kimmie Snow.
- **Website / online systems / technical support:** route to **Ian Deans (ian@skylinksgolf.com)**. [^https://github.com/Skylinks-Golf/skylinks-wordpress-site/blob/main/TEAM_ROSTER.md]

## What to optimize for when assisting Ian (agent success criteria)
- **Operational continuity first:** reliability, clear failure modes, and recovery paths matter more than elegant architecture. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Integration boundaries:** define each system’s responsibilities; avoid “mystery glue” that only works in one person’s head. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Maintainability + handoff:** Ian is long-term owner; deliver artifacts Ian can reopen months later and understand quickly. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- **Documentation as part of the deliverable:** include validation steps, staff-facing impact, and a short runbook for troubleshooting. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

## Common request types you will receive (be ready)
- Booking/tee sheet issues that are “ops-visible” but “system-rooted.”
- Payment/subscription issues (Stripe) with reconciliation requirements.
- WordPress form changes that require backend changes (GCP) and data updates (Airtable).
- POS-related workflow issues where the fix may be process + configuration + documentation (not just code). [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

## Guardrails (do not violate)
- Do not make assumptions about production changes. Prefer reversible changes, staged rollout, and explicit validation steps.
- Treat any access tokens / API keys as sensitive; do not copy or expose secrets in outputs.
- When uncertain, ask for the authoritative system-of-record for that workflow (POS vs Airtable vs WordPress vs GCP). [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]

## Systems and keywords (for searching relevant docs/specs)
- GCP, Cloud Functions, Node.js/Express [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- WordPress forms/components/integrations [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Stripe payments/subscriptions [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Airtable data + automations/webhooks [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Lightspeed Golf / POS + facility tech support [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- NCGA / USGA admin workflows + compliance [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
- Tee sheet administration + online booking workflows [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c][^https://www.notion.so/2a8fe8ecae8f8010974fdb38a360f93d]

## “10 things to remember” (fast recall)
1. Ian is Operations & Systems Administrator; reports to Owner/GM. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
2. Ian is an operator-engineer: ops reality + code delivery. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
3. Technical systems (website/online) route to Ian. [^https://github.com/Skylinks-Golf/skylinks-wordpress-site/blob/main/TEAM_ROSTER.md]
4. Ian owns booking/tee sheet admin. [^https://www.notion.so/2a8fe8ecae8f8010974fdb38a360f93d]
5. Ian owns Stripe/payment system integrity. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
6. Ian builds backend services and GCP deployments. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
7. Ian builds WordPress forms that connect to backend + data stores. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
8. Ian designs/maintains POS ↔ ops data ↔ website integrations. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
9. Ian runs NCGA/USGA club admin and compliance operations. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]
10. The “definition of done” includes runbooks and staff-facing clarity, not just working code. [^https://www.notion.so/bfb5cb1d139342349f6e6e066bbcfb3c]