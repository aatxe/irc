use std::io::{InvalidInput, IoError, IoResult};

fn process(msg: &str) -> IoResult<(&str, &str, Vec<&str>)> {
    let reg = regex!(r"^(?::([^ ]+) )?([^ ]+)(.*)");
    let cap = match reg.captures(msg) {
        Some(x) => x,
        None => return Err(IoError {
            kind: InvalidInput,
            desc: "Failed to parse line",
            detail: None,
        }),
    };
    let source = cap.at(1);
    let command = cap.at(2);
    let args = parse_args(cap.at(3));
    Ok((source, command, args))
}

fn parse_args(line: &str) -> Vec<&str> {
    let reg = regex!(r" ([^: ]+)| :([^\r\n]*)[\r\n]*$");
    reg.captures_iter(line).map(|cap| {
        match cap.at(1) {
            "" => cap.at(2),
            x => x,
        }
    }).collect()
}

#[cfg(test)]
mod test {
    use super::{process, parse_args};

    #[test]
    fn process_line() {
        let res = process(":flare.to.ca.fyrechat.net 353 pickles = #pickles :pickles awe\r\n").unwrap();
        let (source, command, args) = res;
        assert_eq!(source, "flare.to.ca.fyrechat.net");
        assert_eq!(command, "353");
        assert_eq!(args, vec!["pickles", "=", "#pickles", "pickles awe"]);

        let res = process("PING :flare.to.ca.fyrechat.net\r\n").unwrap();
        let (source, command, args) = res;
        assert_eq!(source, "");
        assert_eq!(command, "PING");
        assert_eq!(args, vec!["flare.to.ca.fyrechat.net"]);
    }

    #[test]
    fn process_args() {
        let res = parse_args("PRIVMSG #vana :hi");
        assert_eq!(res, vec!["#vana", "hi"])
    }
}
