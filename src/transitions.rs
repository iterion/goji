//! Interfaces for accessing and managing transition

// Ours
use crate::{Error, Jira, Result, TransitionOption, TransitionOptions, TransitionTriggerOptions};

/// issue transition interface
#[derive(Debug)]
pub struct Transitions {
    jira: Jira,
    key: String,
}

impl Transitions {
    pub fn new<K>(jira: &Jira, key: K) -> Transitions
    where
        K: Into<String>,
    {
        Transitions {
            jira: jira.clone(),
            key: key.into(),
        }
    }

    /// return list of transitions options for this issue
    pub async fn list(&self) -> Result<Vec<TransitionOption>> {
        self.jira
            .get::<TransitionOptions>(
                "api",
                &format!("/issue/{}/transitions?expand=transitions.fields", self.key),
            ).await
            .map(|wrapper| wrapper.transitions)
    }

    /// trigger a issue transition
    /// to transition with a resolution use TransitionTrigger::builder(id).resolution(name)
    pub async fn trigger(&self, trans: TransitionTriggerOptions) -> Result<()> {
        self.jira
            .post::<(), TransitionTriggerOptions>(
                "api",
                &format!("/issue/{}/transitions", self.key),
                trans,
            ).await
            .or_else(|e| match e {
                Error::Serde(_) => Ok(()),
                e => Err(e),
            })
    }
}
