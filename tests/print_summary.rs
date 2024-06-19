use log_parser::runner::Runner;
use rstest::rstest;
use std::{fs::File, io::Cursor, io::Read, path::Path};

#[rstest]
#[case(
    "/tests/data/input-print-summary-basic.txt",
    "/tests/data/output-print-summary-basic.txt"
)]
#[case(
    "/tests/data/input-print-summary-multiple-pipelines.txt",
    "/tests/data/output-print-summary-multiple-pipelines.txt"
)]
fn test_e2e_print_summary(#[case] input_file_path: &str, #[case] expected_output_path: &str) {
    let log_file_path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), input_file_path);
    let stdout_writer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let log_file = Path::new(&log_file_path);

    let mut runner = Runner::builder()
        .with_log_file(log_file.to_str().unwrap())
        .with_stdout_writer(stdout_writer)
        .build()
        .unwrap();

    runner.run().unwrap();

    let actual_output = runner.stdout_writer().clone().into_inner();
    let actual_output_str = String::from_utf8(actual_output).unwrap();

    let expected_output_file_path =
        format!("{}{}", env!("CARGO_MANIFEST_DIR"), expected_output_path);
    let mut expected_output_file = File::open(expected_output_file_path).unwrap();
    let mut expected_output = String::new();
    expected_output_file
        .read_to_string(&mut expected_output)
        .unwrap();

    assert_eq!(expected_output, actual_output_str);
}
