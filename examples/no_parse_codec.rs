//! Creates a IRC Client using the `no-parse-codec`.
//! This means that messages are received as Strings rather than [`irc_proto::Command`] objects.
//! This only works if you run it with `--no-default-features`.

// TODO: This should be an integration test for the `no_parse_codec` crate

use futures::prelude::*;
use irc::client::prelude::*;
use no_parse_codec::*;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
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
    }

    Ok(())
}
