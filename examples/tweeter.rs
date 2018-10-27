extern crate irc;

// NOTE: you can find an asynchronous version of this example with `IrcReactor` in `tooter.rs`.
#[cfg(feature = "client")]
fn main() {
    use std::default::Default;
    use std::thread;
    use std::time::Duration;
    use irc::client::prelude::*;

    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.mozilla.org".to_owned()),
        channels: Some(vec!["#rust-spam".to_owned()]),
        ..Default::default()
    };
    let client = IrcClient::from_config(config).unwrap();
    client.identify().unwrap();
    let client2 = client.clone();
    // Let's set up a loop that just prints the messages.
    thread::spawn(move || {
        client2.stream().map(|m| print!("{}", m)).wait().count();
    });
    loop {
        client.send_privmsg("#rust-spam", "TWEET TWEET").unwrap();
        thread::sleep(Duration::new(10, 0));
    }
}

#[cfg(not(feature = "client"))]
fn main() {
    eprintln!("built without client support")
}
