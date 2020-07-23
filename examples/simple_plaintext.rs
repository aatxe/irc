use futures::prelude::*;
use irc::client::prelude::*;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("chat.freenode.net".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        use_tls: Some(false),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let sender = client.sender();

    while let Some(message) = stream.next().await.transpose()? {
        print!("{}", message);

        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                if msg.contains(client.current_nickname()) {
                    sender.send_privmsg(target, "Hi!")?;
                }
            }
            _ => (),
        }
    }

    Ok(())
}
