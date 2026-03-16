use super::Storage;
use anyhow::Result;
use async_trait::async_trait;
use koad_proto::cass::v1::{EpisodicMemory, FactCard};
use std::sync::Mutex;

#[derive(Default)]
pub struct MockStorage {
    pub facts: Mutex<Vec<FactCard>>,
    pub episodes: Mutex<Vec<EpisodicMemory>>,
}

#[async_trait]
impl Storage for MockStorage {
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        let mut facts = self.facts.lock().unwrap();
        facts.push(fact);
        Ok(())
    }

    async fn query_facts(
        &self,
        domain: &str,
        _tags: &[String],
        limit: u32,
    ) -> Result<Vec<FactCard>> {
        let facts = self.facts.lock().unwrap();
        let result: Vec<FactCard> = facts
            .iter()
            .filter(|f| f.domain == domain)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn query_agent_facts(&self, agent_name: &str, limit: u32) -> Result<Vec<FactCard>> {
        let facts = self.facts.lock().unwrap();
        let result: Vec<FactCard> = facts
            .iter()
            .filter(|f| f.source_agent == agent_name)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()> {
        let mut episodes = self.episodes.lock().unwrap();
        episodes.push(episode);
        Ok(())
    }

    async fn query_recent_episodes(
        &self,
        _agent_name: &str,
        limit: u32,
    ) -> Result<Vec<EpisodicMemory>> {
        let episodes = self.episodes.lock().unwrap();
        let result: Vec<EpisodicMemory> = episodes
            .iter()
            .rev()
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }
}
