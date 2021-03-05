extern crate env_logger;
extern crate goji;

use goji::{Credentials, Jira};
use std::env;
use futures::executor::block_on;

async fn search_example() {
    drop(env_logger::init());
    if let (Ok(host), Ok(user), Ok(pass)) = (
        env::var("JIRA_HOST"),
        env::var("JIRA_USER"),
        env::var("JIRA_PASS"),
    ) {
        let query = env::args().nth(1).unwrap_or("assignee=doug".to_owned());

        let jira = Jira::new(host, Credentials::Basic(user, pass)).unwrap();

        // TODO restore to iter
        match jira.search().list(query, &Default::default()).await {
            Ok(results) => {
                for issue in results.issues {
                    println!(
                        "{} {} ({}): reporter {} assignee {}",
                        issue.key,
                        issue.summary().unwrap_or("???".to_owned()),
                        issue
                            .status()
                            .map(|value| value.name,)
                            .unwrap_or("???".to_owned(),),
                        issue
                            .reporter()
                            .map(|value| value.display_name,)
                            .unwrap_or("???".to_owned(),),
                        issue
                            .assignee()
                            .map(|value| value.display_name,)
                            .unwrap_or("???".to_owned(),)
                    );
                }
            }
            Err(err) => panic!("{:#?}", err),
        }
    }
}

fn main() {
    let example = search_example(); // Nothing is printed
    block_on(example); // `future` is run and "hello, world!" is printed
}
