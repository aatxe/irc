use futures::prelude::*;
use irc::client::prelude::*;
use std::time::Duration;

// NOTE: you can find an asynchronous version of this example with `IrcReactor` in `tooter.rs`.
#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.mozilla.org".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let mut interval = tokio::time::interval(Duration::from_secs(10)).fuse();

    loop {
        futures::select! {
            m = stream.select_next_some() => {
                println!("{}", m?);
            }
            _ = interval.select_next_some() => {
                client.send_privmsg("#rust-spam", "TWEET TWEET")?;
            }
        }
    }
}
