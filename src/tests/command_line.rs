use clap::Arg;
use clap::ArgAction;
use clap::Command;
use std::env::args;

pub struct CommandLine {
    pub failed: bool,
    pub folder_path: String,
    pub tests: Vec<String>,
}
impl CommandLine {
    pub fn parse_args(args: impl Iterator<Item = String>) -> Self {
        let command = Command::new("compliance");
        let matches = command
            .version("1.0")
            .author("@springcomp")
            .about("Runs JMESPath compliance tests suite")
            .arg(
                Arg::new("dir")
                    .short('d')
                    .long("directory")
                    .action(ArgAction::Set)
                    .help("The top-level compliance tests directory"),
            )
            .arg(
                Arg::new("failed")
                    .short('f')
                    .long("failed-tests-only")
                    .action(ArgAction::SetTrue)
                    .help("Displays failed tests only"),
            )
            .arg(
                Arg::new("test")
                    .short('t')
                    .long("compliance-test")
                    .action(ArgAction::Append)
                    .help("Select one or more compliance tests to run"),
            )
            .get_matches_from(args);

        let tests = matches.get_many::<String>("test");
        let dir = matches.get_one::<String>("dir").map(|s| s.as_str());
        let failed = matches.get_flag("failed");

        // select top-level tests folder

        let mut folder_path = "compliance/tests";
        if let Some(path) = dir {
            folder_path = path;
        }

        // select list of test cases

        let tests = match tests {
            Some(v) => v.map(|x| x.clone()).collect(),
            None => vec![],
        };

        CommandLine {
            folder_path: folder_path.to_string(),
            failed,
            tests,
        }
    }
    pub fn parse() -> Self {
        Self::parse_args(args())
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::CommandLine;

    #[rstest]
    #[case("compliance -h")]
    #[case("compliance -h")]
    fn it_displays_help(#[case] _args: &str) {}

    #[rstest]
    #[case("compliance", false)]
    #[case("compliance -f", true)]
    #[case("compliance --failed-tests-only", true)]
    fn it_parses_arg_failed(#[case] args: &str, #[case] expected: bool) {
        let options = args.split(" ").map(|x| x.to_string());
        let command_line = CommandLine::parse_args(options);
        assert_eq!(expected, command_line.failed);
    }

    #[rstest]
    #[case("compliance", "compliance/tests")]
    #[case("compliance -d /tmp", "/tmp")]
    #[case("compliance --directory /tmp", "/tmp")]
    fn it_parses_arg_directory(#[case] args: &str, #[case] expected: &str) {
        let options = args.split(" ").map(|x| x.to_string());
        let command_line = CommandLine::parse_args(options);
        assert_eq!(expected, command_line.folder_path);
    }
}
