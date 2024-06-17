use log_parser::runner::Runner;
use std::{io::Cursor, path::Path};

#[test]
fn test_print_summary_e2e() {
    let log_file_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/log-print-summary.txt"
    );
    let stdout_writer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let log_file = Path::new(log_file_path);

    let mut runner = Runner::builder()
        .with_log_file(log_file.to_str().unwrap())
        .with_stdout_writer(stdout_writer)
        .build()
        .unwrap();

    runner.run().unwrap();

    let actual_output = runner.stdout_writer().clone().into_inner();
    let actual_output_str = String::from_utf8(actual_output).unwrap();
    let expected_output = r###"Pipeline 1:
  Stage 001 - Headers: 10m 40s
  Stage 002 - Bodies: 2h 32m
  Stage 003 - SenderRecovery: 1h 37m
  Stage 004 - Execution: 43h 42m
  Stage 005 - MerkleUnwind: 0s
  Stage 006 - AccountHashing: 2m 59s
  Stage 007 - StorageHashing: 46m 23s
  Stage 008 - MerkleExecute: 47m 12s
  Stage 009 - TransactionLookup: 32m 44s
  Stage 010 - IndexStorageHistory: 3h 32m
  Stage 011 - IndexAccountHistory: 1h 38m
  Stage 012 - Finish: 0s
  Total Pipeline Duration: 55h 23m
Total Aggregate Duration: 55h 23m
"###;

    assert_eq!(expected_output, actual_output_str);
}
