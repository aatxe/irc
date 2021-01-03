use irc::client::prelude::*;
use std::time::Duration;
use tokio_stream::StreamExt as _;

// NOTE: you can find an asynchronous version of this example with `IrcReactor` in `tooter.rs`.
#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("chat.freenode.net".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            Some(m) = stream.next() => {
                println!("{}", m?);
            }
            _ = interval.tick() => {
                client.send_privmsg("#rust-spam", "TWEET TWEET")?;
            }
        }
    }
}
