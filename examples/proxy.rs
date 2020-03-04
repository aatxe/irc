use futures::prelude::*;
use irc::client::data::ProxyType;
use irc::client::prelude::*;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("rust-irc-bot".to_owned()),
        alt_nicks: vec!["bananas".to_owned(), "apples".to_owned()],
        server: Some("irc.oftc.net".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        proxy_type: Some(ProxyType::Socks5),
        proxy_server: Some("127.0.0.1".to_owned()),
        proxy_port: Some(9050),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        print!("{}", message);
    }

    Ok(())
}
