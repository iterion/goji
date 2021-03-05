extern crate env_logger;
extern crate goji;

use goji::{Credentials, Jira, TransitionTriggerOptions};
use std::env;
use futures::executor::block_on;


async fn transitions_example() {
    drop(env_logger::init());
    if let (Ok(host), Ok(user), Ok(pass), Ok(key)) = (
        env::var("JIRA_HOST"),
        env::var("JIRA_USER"),
        env::var("JIRA_PASS"),
        env::var("JIRA_KEY"),
    ) {
        let jira = Jira::new(host, Credentials::Basic(user, pass)).unwrap();

        println!("{:#?}", jira.issues().get(key.clone()).await);
        let transitions = jira.transitions(key);
        for option in transitions.list().await {
            println!("{:#?}", option);
        }
        if let Ok(transition_id) = env::var("JIRA_TRANSITION_ID") {
            transitions
                .trigger(TransitionTriggerOptions::new(transition_id)).await
                .unwrap()
        }
    }
}

fn main() {
    let example = transitions_example(); // Nothing is printed
    block_on(example); // `future` is run and "hello, world!" is printed
}
