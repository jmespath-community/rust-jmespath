pub mod compliance;
use clap::App;
use clap::Arg;

use crate::compliance::Compliance;
use crate::compliance::ComplianceReport;

fn main() {

    let matches = App::new("compliance")
        .version("1.0")
        .author("@springcomp")
        .about("Runs JMESPath compliance tests suite")
        .arg(
            Arg::with_name("dir")
                .short("d")
                .long("directory")
                .takes_value(true)
                .help("The top-level compliance tests directory"))
        .arg(
            Arg::with_name("failed")
                .short("f")
                .long("failed-tests-only")
                .takes_value(false)
                .help("Displays failed tests only"))
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("compliance-test")
                .multiple(true)
                .takes_value(true)
                .help("Select one or more compliance tests to run"))
        .get_matches();

    let tests = matches.values_of("test");
    let dir = matches.value_of("dir");
    let failed = matches.is_present("failed");

    // select top-level tests folder

    let mut folder_path =
        Compliance::get_full_path(&Compliance::get_current_path(), &"compliance/tests");
    if let Some(path) = dir {
        folder_path = path.to_string();
    }

    println!("Running compliance tests suite from '{}'…", folder_path);

    // restrict the tests to those selected by the user

    let mut paths = Compliance::get_compliance_test_files(&folder_path);
    if let Some(v) = tests {
        let selected = v.collect::<Vec<&str>>();
        paths = paths.into_iter().filter(|p| selected.iter().any(|s| p.contains(s))).collect::<Vec<String>>();
    }

    let mut report = ComplianceReport {
        test_cases: 0,
        succeeded: 0,
        failed: 0,
    };

    for path in paths {
        let current = Compliance::run_compliance_test_suite(&path, failed);
        report.test_cases = report.test_cases + current.test_cases;
        report.succeeded = report.succeeded + current.succeeded;
        report.failed = report.failed + current.failed;
    }

    println!("Compliance tests suites ran successfully.");
    println!("{:?}", report);
}
