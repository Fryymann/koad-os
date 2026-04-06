use super::{PulseTier, Storage};
use anyhow::Result;
use async_trait::async_trait;
use koad_proto::cass::v1::{EpisodicMemory, FactCard, Pulse};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;

pub struct MockStorage {
    pub facts: Arc<Mutex<Vec<FactCard>>>,
    pub episodes: Arc<Mutex<Vec<EpisodicMemory>>>,
}

pub struct MockPulseStore {
    pulses: Arc<TokioMutex<Vec<Pulse>>>,
}

impl MockPulseStore {
    pub fn new() -> Self {
        Self {
            pulses: Arc::new(TokioMutex::new(vec![])),
        }
    }

    pub async fn seed(&self, pulse: Pulse) {
        self.pulses.lock().await.push(pulse);
    }

    pub async fn all(&self) -> Vec<Pulse> {
        self.pulses.lock().await.clone()
    }
}

#[async_trait]
impl PulseTier for MockPulseStore {
    async fn add_pulse(&self, pulse: Pulse) -> Result<()> {
        self.pulses.lock().await.push(pulse);
        Ok(())
    }

    async fn get_active_pulses(&self, _role: &str) -> Result<Vec<Pulse>> {
        Ok(self.pulses.lock().await.clone())
    }
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            facts: Arc::new(Mutex::new(Vec::new())),
            episodes: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Storage for MockStorage {
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        let mut facts = self.facts.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        facts.push(fact);
        Ok(())
    }

    async fn query_facts(
        &self,
        _domain: &str,
        _tags: &[String],
        _limit: u32,
    ) -> Result<Vec<FactCard>> {
        let facts = self.facts.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        Ok(facts.clone())
    }

    async fn query_agent_facts(
        &self,
        _agent_name: &str,
        _limit: u32,
        _task_id: Option<&str>,
    ) -> Result<Vec<FactCard>> {
        let facts = self.facts.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        Ok(facts.clone())
    }

    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()> {
        let mut episodes = self.episodes.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        episodes.push(episode);
        Ok(())
    }

    async fn query_recent_episodes(
        &self,
        _agent_name: &str,
        _limit: u32,
        _task_id: Option<&str>,
    ) -> Result<Vec<EpisodicMemory>> {
        let episodes = self.episodes.lock().map_err(|_| anyhow::anyhow!("Mutex poisoned"))?;
        Ok(episodes.clone())
    }
}
