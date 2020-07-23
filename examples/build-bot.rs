extern crate irc;

use futures::prelude::*;
use irc::client::prelude::*;
use std::default::Default;
use std::env;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let repository_slug = env::var("TRAVIS_REPO_SLUG").unwrap();
    let branch = env::var("TRAVIS_BRANCH").unwrap();
    let commit = env::var("TRAVIS_COMMIT").unwrap();
    let commit_message = env::var("TRAVIS_COMMIT_MESSAGE").unwrap();
    let features = env::var("FEATURES").unwrap();

    let config = Config {
        nickname: Some("irc-crate-ci".to_owned()),
        server: Some("chat.freenode.net".to_owned()),
        alt_nicks: vec!["[irc-crate-ci]".to_owned()],
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;

    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::Response(Response::RPL_ISUPPORT, _) => {
                client.send_privmsg(
                    "#commits",
                    format!(
                        "[{}/{}] ({}) {} [{}]",
                        repository_slug,
                        branch,
                        &commit[..7],
                        commit_message,
                        features,
                    ),
                )?;

                client.send_quit("QUIT")?;
            }
            _ => (),
        }
    }

    Ok(())
}
