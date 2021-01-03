use irc::client::prelude::*;
use std::time::Duration;

// NOTE: this example is a conversion of `tweeter.rs` to an asynchronous style with `IrcReactor`.
#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("mastodon".to_owned()),
        server: Some("chat.freenode.net".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        ..Default::default()
    };

    let client = Client::from_config(config).await?;
    let sender = client.sender();

    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        let _ = interval.tick().await;
        sender.send_privmsg("#rust-spam", "AWOOOOOOOOOO")?;
    }
}
