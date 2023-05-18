use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use jmespath::Value;
use serde::Deserialize;
use serde_json::Value as JsonValue;

/// Represents a collection of [`ComplianceTest`] objects.
#[derive(Debug, Deserialize)]
pub struct ComplianceTestCase {
    pub given: JsonValue,
    pub cases: Vec<ComplianceTest>,
}

/// Represents a single compliance test.
#[derive(Debug, Deserialize)]
pub struct ComplianceTest {
    pub expression: String,
    pub result: Option<JsonValue>,
    pub error: Option<String>,
}

/// Captures the result of a running a [`ComplianceTest`].
#[derive(Debug)]
pub enum ComplianceResult {
    /// Evaluating an expression succeeded and the result was expected.
    Succeeded,
    /// Evaluating an expression succeeded but the result was not expected.
    ComparisonFailed,
    /// Evaluating an expression failed with an unexpected error type.
    UnexpectedError,
}

#[derive(Debug, Copy, Clone)]
pub struct ComplianceReport {
    pub test_cases: usize,
    pub succeeded: usize,
    pub failed: usize,
}
impl ComplianceReport {
    pub fn succeeded(&self) -> bool {
        self.succeeded == self.test_cases
    }
}

/// Represents helpers to iterate and run compliance tests.
pub struct Compliance {}
impl Compliance {
    /// Runs a compliance test suite.
    pub fn run_compliance_test_suite(
        path: &str,
        display_failed_tests_only: bool,
    ) -> ComplianceReport {
        let name = Path::new(path);
        if let Some(file_name) = name.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                println!("Running compliance tests '{}'.", file_name_str);
            }
        }

        let mut report = ComplianceReport {
            test_cases: 0,
            succeeded: 0,
            failed: 0,
        };

        let suites = Compliance::load_compliance_test_suite(path);

        for suite in suites {
            for case in suite.cases {
                let result = Compliance::run_compliance_test_case(
                    &suite.given,
                    &case.expression,
                    case.result,
                    &case.error,
                    display_failed_tests_only,
                );
                report.test_cases = report.test_cases + 1;
                if let ComplianceResult::Succeeded = result {
                    report.succeeded = report.succeeded + 1;
                } else {
                    report.failed = report.failed + 1;
                }
            }
        }

        report
    }
    /// Runs a single test case and reports results.
    pub fn run_compliance_test_case(
        given: &JsonValue,
        expression: &str,
        expected: Option<JsonValue>,
        error: &Option<String>,
        display_failed_tests_only: bool,
    ) -> ComplianceResult {
        let given_value = jmespath::Value::map_from_json(given);
        let found = jmespath::search(&expression, &given_value).map_err(|e| e.kind);
        //println!("given_value: {}, expression: {}, found: {:?}", given_value, expression, found);
        match found {
            Ok(actual_value) => {
                //println!("expected: {:?}", expected);
                if let Some(result) = expected {
                    let expected_value = jmespath::Value::map_from_json(&result);
                    if expected_value == actual_value {
                        if !display_failed_tests_only {
                            println!("{} => ✔️.", expression);
                        }
                        return ComplianceResult::Succeeded;
                    }
                    println!("{} => ❌️.", expression);
                    return ComplianceResult::ComparisonFailed;
                }
                if let Some(err) = error {
                    println!(
                        "{} => evaluation succeeded whereas error '{}' was expected.",
                        expression, err
                    );
                    return ComplianceResult::UnexpectedError;
                } else {
                    let expected_value = Value::Null;
                    if expected_value == actual_value {
                        if !display_failed_tests_only {
                            println!("{} => ✔️.", expression);
                        }
                        return ComplianceResult::Succeeded;
                    }
                    println!("{} => ❌️.", expression);
                    return ComplianceResult::ComparisonFailed;
                }
            }
            Err(kind) => {
                if let Some(err) = error {
                    if format!("{}", kind) == *err {
                        if !display_failed_tests_only {
                            println!("{} => ✔️.", expression);
                        }
                        return ComplianceResult::Succeeded;
                    }
                    println!("{} => ❌ evaluation failed with error '{}' whereas '{}' was expected instead.", expression, kind, err);
                    return ComplianceResult::UnexpectedError;
                }
                if let Some(_) = expected {
                    println!(
                        "{} => ❌ evaluation failed with unexpected error '{}'.",
                        expression, kind
                    );
                    return ComplianceResult::UnexpectedError;
                } else {
                    println!("{} => INVALID TEST ⚠️", expression);
                    return ComplianceResult::UnexpectedError;
                }
            }
        }
    }
    /// Load the contents of a compliance test suite in memory.
    pub fn load_compliance_test_suite(path: &str) -> Vec<ComplianceTestCase> {
        let err = format!("Failed to open file '{}'", path);
        let file = File::open(path).expect(&err);
        let reader = BufReader::new(file);
        let suite: Vec<ComplianceTestCase> =
            serde_json::from_reader(reader).expect("Failed to parse JSON");

        suite
    }
    /// Returns the paths to the currently active compliance tests
    pub fn get_compliance_test_files(folder_path: &str) -> Vec<String> {
        let excluded = Self::get_excluded_files();
        let compliance = Self::walk_compliance_test_files(folder_path);

        let mut included = Vec::new();
        for path in compliance {
            if !excluded.iter().any(|x| path.contains(x)) {
                included.push(path)
            }
        }

        included
    }

    /// Returns the relative paths of currently excluded compliance tests
    fn get_excluded_files() -> Vec<String> {
        let mut excluded = Vec::new();
        excluded.push("legacy/legacy-literal.json".to_string());
        excluded.push("benchmarks.json".to_string());
        excluded.push("function_group_by.json".to_string());
        excluded.push("functions.json".to_string());
        excluded.push("functions_strings.json".to_string());
        excluded
    }

    fn walk_compliance_test_files(folder_path: &str) -> Vec<String> {
        let mut paths: Vec<String> = vec![];

        // Read the directory contents
        let entries = fs::read_dir(folder_path).expect("Failed to read directory");

        // Iterate over the directory entries
        for entry in entries {
            if let Ok(entry) = entry {
                // Get the file path
                let os_path = entry.path();
                let full_path = Self::get_full_path(folder_path, &os_path);

                // Walk into subfolder
                let file_type = entry.file_type();
                if let Ok(file_type) = file_type {
                    if file_type.is_dir() {
                        let collection = Self::walk_compliance_test_files(&full_path);
                        paths.extend(collection);
                    }
                }

                // Check if the file has the .json extension
                if let Some(extension) = os_path.extension() {
                    if extension == "json" {
                        // Check if the file is excluded
                        if let Some(_) = os_path.file_name() {
                            paths.push(full_path);
                        }
                    }
                }
            }
        }
        paths
    }
    pub fn get_full_path<P>(folder_path: &str, os_path: &P) -> String
    where
        P: AsRef<Path>,
    {
        Path::new(folder_path)
            .join(os_path)
            .to_string_lossy()
            .to_string()
    }
    pub fn get_current_path() -> String {
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
