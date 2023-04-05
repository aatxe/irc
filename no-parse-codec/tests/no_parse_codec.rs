//! Creates a IRC Client using the `no-parse-codec`.
//! This means that messages are received as Strings rather than [`irc_proto::Command`] objects.
//! This only works if you run it with `--no-default-features`.

#[allow(unused_imports)] // avoid false-positive in next line
use futures_util::StreamExt;

use irc::client::prelude::*;
use no_parse_codec::*;

#[tokio::test]
async fn connect_to_server() -> irc::error::Result<()> {
    env_logger::init();
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("chat.freenode.net".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        ..Default::default()
    };

    let mut client = Client::<NoParseCodec>::from_config_with_codec(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        print!("{}", message);
        if message.to_string().contains("End of /NAMES list.") {
            return Ok(());
        }
    }

    panic!("Failed to maintain connection");
}
