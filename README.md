# irc [![Build Status](https://travis-ci.org/aatxe/irc.svg?branch=master)](https://travis-ci.org/aatxe/irc) #
A thread-safe IRC library in Rust based on iterators. It's hopefully compliant with 
[RFC 2812](http://tools.ietf.org/html/rfc2812). You can find up-to-date, ready-to-use documentation
 online [here](http://www.rust-ci.org/aatxe/irc/doc/irc/). The documentation is generated 
with the default features. These are, however, strictly optional and can be disabled by the
end-user. 

## Getting Started ##

To start using this library with cargo, you can simply add `irc = "*"` to your dependencies to your
Cargo.toml file. From there, you can look to the examples and the documentation to see how to
proceed. Making a simple bot is easy though:

```rust
extern crate irc;

use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

fn main() {
    let irc_server = IrcServer::new("config.json").unwrap();
    let server = Wrapper::new(&irc_server);
    server.identify().unwrap();
    for message in server.iter() {
        // Do message processing.
    }
}
```

## Contributing ##
Contributions to this library would be immensely appreciated. As this project is public domain, 
all prospective contributors must 
[sign the Contributor License Agreement](https://www.clahub.com/agreements/aatxe/irc), a 
public domain dedication.
