use futures::prelude::*;
use irc::client::prelude::*;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        alt_nicks: vec!["bananas".to_owned(), "apples".to_owned()],
        server: Some("irc.mozilla.org".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    loop {
        let message = stream.select_next_some().await?;

        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if msg.contains("pickles") {
                client.send_privmsg(target, "Hi!").unwrap();
            }
        }
    }
}
