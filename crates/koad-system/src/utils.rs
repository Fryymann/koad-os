use anyhow::{Context, Result};
use fred::interfaces::{KeysInterface, LuaInterface};
use koad_core::config::KoadConfig;
use koad_core::db::KoadDB;
use koad_core::utils::lock::DistributedLock;
use koad_core::utils::redis::RedisClient;
use rusqlite::params;
use std::env;
use std::path::PathBuf;

pub struct RedisLockClient {
    pub socket: PathBuf,
}

#[async_trait::async_trait]
impl DistributedLock for RedisLockClient {
    async fn lock(&self, sector: &str, agent_name: &str, ttl_secs: u64) -> Result<bool> {
        let client =
            RedisClient::new(&self.socket.parent().unwrap().to_string_lossy(), false).await?;
        let key = format!("koad:lock:{}", sector);
        let session_id = env::var("KOAD_SESSION_ID").unwrap_or_else(|_| agent_name.to_string());
        let val = format!("{}:{}", agent_name, session_id);

        let res: Option<String> = client
            .pool
            .set(
                &key,
                &val,
                Some(fred::types::Expiration::EX(ttl_secs as i64)),
                Some(fred::types::SetOptions::NX),
                false,
            )
            .await?;

        Ok(res.is_some()
            || client.pool.get::<Option<String>, _>(&key).await?.as_deref() == Some(&val))
    }

    async fn unlock(&self, sector: &str, agent_name: &str) -> Result<bool> {
        let client =
            RedisClient::new(&self.socket.parent().unwrap().to_string_lossy(), false).await?;
        let key = format!("koad:lock:{}", sector);
        let session_id = env::var("KOAD_SESSION_ID").unwrap_or_else(|_| agent_name.to_string());
        let val = format!("{}:{}", agent_name, session_id);

        let script = r"
            if redis.call('get', KEYS[1]) == ARGV[1] then
                return redis.call('del', KEYS[1])
            else
                return 0
            end
        ";

        let result: i32 = client
            .pool
            .next()
            .eval(script, vec![key], vec![val])
            .await?;
        Ok(result == 1)
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_issue(
    config: &KoadConfig,
    db: &KoadDB,
    template: &str,
    title: &str,
    weight: &str,
    objective: Option<String>,
    scope: Option<String>,
    labels: Vec<String>,
    raw_body: Option<String>,
) -> Result<koad_board::issue::Issue> {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_ctx = config.resolve_project_context(&current_dir);
    let project = project_ctx.as_ref().map(|(_, p)| p);

    // Resolve repository from Context or DB
    let (owner, repo) = if let Some(p) = project {
        (
            config.get_github_owner(Some(p)),
            config.get_github_repo(Some(p)),
        )
    } else if let Ok(conn) = db.get_conn() {
        let abs_current = std::fs::canonicalize(&current_dir).unwrap_or(current_dir);
        let search_path = abs_current.to_string_lossy().to_string();
        let mut stmt = conn.prepare("SELECT github_repo FROM projects WHERE ?1 LIKE path || '%' ORDER BY length(path) DESC LIMIT 1")?;
        let repo_full: Option<String> = stmt.query_row(params![search_path], |r| r.get(0)).ok();

        if let Some(full) = repo_full {
            let parts: Vec<&str> = full.split('/').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (
                    config.get_github_owner(None::<&str>),
                    config.get_github_repo(None::<&str>),
                )
            }
        } else {
            (
                config.get_github_owner(None::<&str>),
                config.get_github_repo(None::<&str>),
            )
        }
    } else {
        (
            config.get_github_owner(None::<&str>),
            config.get_github_repo(None::<&str>),
        )
    };

    let token = config.resolve_gh_token(project.as_ref().map(|s| s.as_str()), None)?;
    let client = koad_board::GitHubClient::new(token, owner.clone(), repo.clone())?;

    let body = if let Some(rb) = raw_body {
        rb
    } else {
        let template_path = config
            .home
            .join("templates")
            .join("issues")
            .join(format!("{}.md", template));
        if !template_path.exists() {
            anyhow::bail!("Template '{}' not found at {:?}", template, template_path);
        }

        let mut b = std::fs::read_to_string(&template_path)?;

        // String Substitution for fast-spawning
        b = b.replace("[trivial | standard | complex]", weight);

        if let Some(obj) = objective {
            b = b.replace(
                "[Describe the high-level goal of this architectural change]",
                &obj,
            );
            b = b.replace("[Describe the system subsystem to be hardened]", &obj);
            b = b.replace("[Identify the resource or latency bottleneck]", &obj);
            b = b.replace(
                "[Describe the observed behavior vs expected behavior]",
                &obj,
            );
        }
        if let Some(sc) = scope {
            b = b.replace(
                "- [Component A]\n- [Component B]\n- [Interface change/Addition]",
                &sc,
            );
            b = b.replace("- [Recovery logic for X]\n- [Watchdog implementation for Y]\n- [Self-healing procedure for Z]", &sc);
            b = b.replace(
                "- [Caching strategy]\n- [Refactor of inefficient loop]\n- [Payload reduction]",
                &sc,
            );
        }
        b
    };

    client.create_issue(title, &body, labels).await
}
