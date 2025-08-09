pub mod command_line;
pub mod compliance;

use crate::command_line::CommandLine;
use crate::compliance::Compliance;
use crate::compliance::ComplianceReport;

fn main() {
    let command_line = CommandLine::parse();
    let tests = command_line.tests;
    let failed = command_line.failed;
    let mut folder_path = command_line.folder_path;

    println!("{:?}", failed);
    println!("{:?}", folder_path);
    println!("{:?}", Compliance::get_current_path());

    folder_path = folder_path.replace("/", std::path::MAIN_SEPARATOR_STR);
    let folder_path = Compliance::get_full_path(&Compliance::get_current_path(), &folder_path);

    println!("Running compliance tests suite from '{}'…", folder_path);

    // restrict the tests to those selected by the user

    let mut paths = Compliance::get_compliance_test_files(&folder_path);
    if tests.len() > 0 {
        paths = paths
            .into_iter()
            .filter(|p| tests.iter().any(|s| p.contains(s)))
            .collect::<Vec<String>>();
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
