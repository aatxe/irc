# irc [![Build Status](https://travis-ci.org/aatxe/irc.svg?branch=master)](https://travis-ci.org/aatxe/irc) [![Crates.io](https://img.shields.io/crates/v/irc.svg)](https://crates.io/crates/irc) [![Built with Spacemacs](https://cdn.rawgit.com/syl20bnr/spacemacs/442d025779da2f62fc86c2082703697714db6514/assets/spacemacs-badge.svg)](http://spacemacs.org) #
A robust, thread-safe and async-friendly IRC library in Rust. The client portion is compliant with
[RFC 2812](http://tools.ietf.org/html/rfc2812), [IRCv3.1](http://ircv3.net/irc/3.1.html),
[IRCv3.2](http://ircv3.net/irc/3.2.html), and includes some additional, common features. It also
features a number of useful built-in features for building a powerful client quickly. The
server portion does not exist yet, but hopefully will in the future. You can find up-to-date, ready-to-use documentation
online [here](http://aatxe.github.io/irc/irc/). The documentation is generated with the default
features. These are, however, strictly optional and can be disabled accordingly.

## Getting Started ##

To start using this library with cargo, you can simply add `irc = "0.11.0"` to your dependencies to
your Cargo.toml file. You'll likely want to take a look at some of the examples, as well as the
documentation. You'll also be able to find a small template to get a feel for the library.

## Getting Started by Example ##

```rust
extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

fn main() {
    let cfg = Config {
        nickname: Some(format!("irc-rs")),
        server: Some(format!("irc.example.com")),
        channels: Some(vec![format!("#test")]),
        .. Default::default()
    };
    let server = IrcServer::from_config(cfg).unwrap();
    server.identify().unwrap();
    server.for_each_incoming(|message| {
        // Do message processing.
    })
}
```

It may not seem like much, but all it takes to get started with an IRC connection is the stub
above. In just a few lines, you can be connected to a server and processing IRC messages as you
wish. The library is built with flexibility in mind. If you need to work on multiple threads,
simply clone the server and have at it. We'll take care of the rest.

You'll probably find that programmatic configuration is a bit of a chore, and you'll often want to
be able to change the configuration between runs of the program (for example, to change the server
that you're connecting to). Fortunately, runtime configuration loading is straightforward.

```rust
extern crate irc;

use irc::client::prelude::*;

fn main() {
    let server = IrcServer::new("config.json").unwrap();
    server.identify().unwrap();
    server.for_each_incoming(|message| {
        // Do message processing.
    })
}
```

## Configuration ##

Like the rest of the IRC crate, configuration is built with flexibility in mind. You can easily
create `Config` objects programmatically and choose your own methods for handling any saving or
loading of configuration required. However, for convenience, we've also included the option of
loading JSON files with `serde` to write configurations. All the fields are optional, and thus
any of them can be omitted (though, omitting a nickname or server will cause the program to fail
for obvious reasons). That being said, here's an example of a complete configuration:

```json
{
  "owners": [],
  "nickname": "user",
  "nick_password": "password",
  "alt_nicks": ["user_", "user__"],
  "username": "user",
  "realname": "Test User",
  "server": "chat.freenode.net",
  "port": 6697,
  "password": "",
  "use_ssl": true,
  "cert_path": "cert.der",
  "encoding": "UTF-8",
  "channels": ["#rust", "#haskell", "#fake"],
  "channel_keys": {
    "#fake": "password"
  },
  "umodes": "+RB-x",
  "user_info": "I'm a test user for the Rust IRC crate.",
  "version": "irc:git:Rust",
  "source": "https://github.com/aatxe/irc",
  "ping_time": 180,
  "ping_timeout": 10,
  "should_ghost": false,
  "ghost_sequence": [],
  "options": {
    "key": "value",
    "note": "anything you want can be in here!",
    "and": "you can use it to build your own additional configuration options."
  }
}

```

## Contributing ##
Contributions to this library would be immensely appreciated. It should be noted that as this is a
public domain project, any contributions will thus be released into the public domain as well.
