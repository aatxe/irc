#[deriving(Clone, PartialEq, Show)]
pub struct Message {
    pub prefix: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    pub suffix: Option<String>,
}

impl Message {
    pub fn new(prefix: Option<&str>, command: &str, args: Option<Vec<&str>>, suffix: Option<&str>)
        -> Message {
        Message {
            prefix: prefix.map(|s| s.into_string()),
            command: command.into_string(),
            args: args.map_or(Vec::new(), |v| v.iter().map(|s| s.into_string()).collect()),
            suffix: suffix.map(|s| s.into_string()),
        }
    }

    pub fn into_string(&self) -> String {
        let mut ret = String::new();
        if let Some(ref prefix) = self.prefix {
            ret.push(':');
            ret.push_str(prefix[]);
            ret.push(' ');
        }
        ret.push_str(self.command[]);
        for arg in self.args.iter() {
            ret.push(' ');
            ret.push_str(arg[]);
        }
        if let Some(ref suffix) = self.suffix {
            ret.push_str(" :");
            ret.push_str(suffix[]);
        }
        ret
    }
}
