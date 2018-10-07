extern crate irc;

use irc::client::prelude::*;
use std::default::Default;
use std::env;

fn main() {
    let repository_slug = env::var("TRAVIS_REPO_SLUG").unwrap();
    let branch = env::var("TRAVIS_BRANCH").unwrap();
    let commit = env::var("TRAVIS_COMMIT").unwrap();
    let commit_message = env::var("TRAVIS_COMMIT_MESSAGE").unwrap();

    let config = Config {
        nickname: Some("irc-crate-ci".to_owned()),
        server: Some("irc.pdgn.co".to_owned()),
        use_ssl: Some(true),
        ..Default::default()
    };

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, move |client, message| {
        match message.command {
            Command::Response(Response::RPL_ISUPPORT, _, _) => {
                client.send_privmsg(
                    "#commits",
                    format!(
                        "Hello from Travis CI! {}/{} {}: {}",
                        repository_slug, branch, commit, commit_message
                    ),
                )?;
                client.send_quit("QUIT")?;
            }
            _ => (),
        };

        Ok(())
    });

    reactor.run().unwrap();
}
