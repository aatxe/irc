extern crate irc;
extern crate tokio_timer;

use std::default::Default;
use std::time::Duration;
use irc::client::prelude::*;
use irc::error::IrcError;

// NOTE: this example is a conversion of `tweeter.rs` to an asynchronous style with `IrcReactor`.
fn main() {
    let config = Config {
        nickname: Some("mastodon".to_owned()),
        server: Some("irc.mozilla.org".to_owned()),
        channels: Some(vec!["#rust-spam".to_owned()]),
        ..Default::default()
    };

    // We need to create a reactor first and foremost
    let mut reactor = IrcReactor::new().unwrap();
    // and then create a client via its API.
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    // Then, we identify
    client.identify().unwrap();
    // and clone just as before.
    let send_client = client.clone();

    // Rather than spawn a thread that reads the messages separately, we register a handler with the
    // reactor. just as in the original version, we don't do any real handling and instead just
    // print the messages that are received.
    reactor.register_client_with_handler(client, |_, message| {
        print!("{}", message);
        Ok(())
    });

    // We construct an interval using a wheel timer from tokio_timer. This interval will fire every
    // ten seconds (and is roughly accurate to the second).
    let send_interval = tokio_timer::wheel()
        .tick_duration(Duration::from_secs(1))
        .num_slots(256)
        .build()
        .interval(Duration::from_secs(10));

    // And then spawn a new future that performs the given action each time it fires.
    reactor.register_future(send_interval.map_err(IrcError::Timer).for_each(move |()| {
        // Anything in here will happen every 10 seconds!
        send_client.send_privmsg("#rust-spam", "AWOOOOOOOOOO")
    }));

    // Then, on the main thread, we finally run the reactor which blocks the program indefinitely.
    reactor.run().unwrap();
}
