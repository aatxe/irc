echo "{\"owners\": [\"test\"],\"nickname\": \"test\",\"username\": \"test\",\"realname\": \"test\",\"password\": \"\",\"server\": \"irc.test.net\",\"port\": 6667,\"use_ssl\": false,\"encoding\": \"UTF-8\",\"channels\": [\"#test\", \"#test2\"],\"umodes\": \"+BR\",\"options\": {}}" > client_config.json
cargo run --example convertconf --features "json yaml" -- -i client_config.json -o client_config.toml
cargo run --example convertconf --features "json yaml" -- -i client_config.json -o client_config.yaml
