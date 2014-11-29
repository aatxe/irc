# irc [![Build Status](https://travis-ci.org/aaronweiss74/irc.svg?branch=master)](https://travis-ci.org/aaronweiss74/irc) #
A thread-safe IRC library in Rust based on iterators. It's hopefully compliant with [RFC 2812](http://tools.ietf.org/html/rfc2812). You can find up-to-date, pre-made documentation online [here](http://www.rust-ci.org/aaronweiss74/irc/doc/irc/). It's probably worth noting that because of [this upstream issue](https://github.com/sfackler/rust-openssl/issues/6), reading and writing both block together. If this issue is resolved, this will be changed to have reading and writing be completely separate mutexes.

## Contributing ##
Contributions to this library would be immensely appreciated. As this project is public domain, all prospective contributors must [sign the Contributor License Agreement](https://www.clahub.com/agreements/aaronweiss74/irc), a public domain dedication.
