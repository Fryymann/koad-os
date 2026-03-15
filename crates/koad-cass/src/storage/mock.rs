use crate::storage::Storage;
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
        self.facts.lock().unwrap().push(fact);
        Ok(())
    }

    async fn query_facts(
        &self,
        domain: &str,
        _tags: &[String],
        _limit: u32,
    ) -> Result<Vec<FactCard>> {
        let facts = self.facts.lock().unwrap();
        Ok(facts
            .iter()
            .filter(|f| f.domain == domain)
            .cloned()
            .collect())
    }

    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()> {
        self.episodes.lock().unwrap().push(episode);
        Ok(())
    }
}
