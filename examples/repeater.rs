use futures::prelude::*;
use irc::client::prelude::*;

#[tokio::main]
async fn main() -> irc::error::Result<()> {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("chat.freenode.net".to_owned()),
        channels: vec!["#rust-spam".to_owned()],
        burst_window_length: Some(4),
        max_messages_in_burst: Some(4),
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let sender = client.sender();

    loop {
        let message = stream.select_next_some().await?;

        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if msg.starts_with(&*client.current_nickname()) {
                let tokens: Vec<_> = msg.split(' ').collect();
                if tokens.len() > 2 {
                    let n = tokens[0].len() + tokens[1].len() + 2;
                    if let Ok(count) = tokens[1].parse::<u8>() {
                        for _ in 0..count {
                            sender.send_privmsg(
                                message.response_target().unwrap_or(target),
                                &msg[n..],
                            )?;
                        }
                    }
                }
            }
        }
    }
}
