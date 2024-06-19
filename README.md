# reth-log-parser

Parses reth's logs and shows several stats.

# Execution

Currently running the tool shows the time taken by reth for doing an historical
sync, specifying the time spent on pipelines and the stages on each of them. You
need to specify the path to the log file on the command line invocation:

```shell
$ cargo run -- ./reth.log
Pipeline 1:
Stage 001 - Headers: 24m 11s
Stage 002 - Bodies: 2h 26m
Stage 003 - SenderRecovery: 1h 5m
Stage 004 - Execution: 41h 14m
Stage 005 - MerkleUnwind: 0s
Stage 006 - AccountHashing: 2m 37s
Stage 007 - StorageHashing: 30m 2s
Stage 008 - MerkleExecute: 24m 12s
Stage 009 - TransactionLookup: 25m 51s
Stage 010 - IndexStorageHistory: 10s
Stage 011 - IndexAccountHistory: 3s
Stage 012 - Finish: 0s
Total Pipeline Duration: 46h 25m
Pipeline 2:
Stage 001 - Headers: 2s
Stage 002 - Bodies: 10s
Stage 003 - SenderRecovery: 3s
Stage 004 - Execution: 5m 18s
Stage 005 - MerkleUnwind: 0s
Stage 006 - AccountHashing: 35s
Stage 007 - StorageHashing: 1m 3s
Stage 008 - MerkleExecute: 33m 7s
Stage 009 - TransactionLookup: 24s
Stage 010 - IndexStorageHistory: 3s
Stage 011 - IndexAccountHistory: 1s
Stage 012 - Finish: 0s
Total Pipeline Duration: 40m 50s
Pipeline 3:
Stage 001 - Headers: 0s
Stage 002 - Bodies: 0s
Stage 003 - SenderRecovery: 0s
Stage 004 - Execution: 4s
Stage 005 - MerkleUnwind: 0s
Stage 006 - AccountHashing: 0s
Stage 007 - StorageHashing: 0s
Stage 008 - MerkleExecute: 4s
Stage 009 - TransactionLookup: 0s
Stage 010 - IndexStorageHistory: 0s
Stage 011 - IndexAccountHistory: 0s
Stage 012 - Finish: 0s
Total Pipeline Duration: 10s
Total Aggregate Duration: 47h 14m
```

## Tests

```shell
$ cargo nextest run  --locked
```
