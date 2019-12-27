use futures::prelude::*;
use irc::client::prelude::*;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.mozilla.org".to_owned()),
        channels: Some(vec!["#rust-spam".to_owned()]),
        use_ssl: Some(true),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    let mut stream = client.stream()?;
    let sender = client.sender();

    loop {
        let message = stream.select_next_some().await?;

        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if msg.contains("pickles") {
                sender.send_privmsg(target, "Hi!")?;
            }
        }
    }
}
