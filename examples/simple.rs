use futures::prelude::*;
use irc::client::prelude::*;

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
    let sender = client.sender();

    while let Some(message) = stream.next().await.transpose()? {
        print!("{}", message);

        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                if msg.contains(client.current_nickname()) {
                    #[cfg(feature = "essentials")]
                    sender.send_privmsg(target, "Hi!")?;
                    #[cfg(not(feature = "essentials"))]
                    sender.send(
                        <irc_proto::Message as irc::client::data::codec::InternalIrcMessageOutgoing>::new_raw(
                            "PRIVMSG".to_owned(),
                            vec![target.to_owned(), "Hi!".to_owned()],
                        ),
                    )?;
                }
            }
            _ => (),
        }
    }

    Ok(())
}
