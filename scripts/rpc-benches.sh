#!/bin/bash

BENCHES_ID="reth-1.0-rc1"
OUTPUT_BASE_PATH="/mnt/data/cryo"

run_bench() {
    local bench_name=${1}
    local bench_command=${2}
    cryo ${bench_command} \
         --max-concurrent-requests 100 \
         --rpc http://localhost:8545 \
         --overwrite \
         --output-dir ${OUTPUT_BASE_PATH}/reports > ${OUTPUT_BASE_PATH}/${BENCHES_ID}.${bench_name}.log
}

run_bench_blocks_to_tip(){
    run_bench blocks_to_tip "blocks -b :20033500"
}

run_bench_blocks_range(){
    run_bench blocks_range "blocks -b 19033400:20033500"
}

run_bench_tx_range1(){
    run_bench tx_range1 "transactions -b 19793400:20033500"
}

run_bench_tx_range2(){
    run_bench tx_range2 "transactions -b 20023500:20033500"
}

run_bench_logs_range1(){
    run_bench logs_range1 "logs -b 18793400:19793400"
}

run_bench_logs_range2(){
    run_bench logs_range2 "logs -b 19793400:20033500"
}

run_bench_state_reads(){
    run_bench traces_state_reads "state_reads -b 20033436:20033500"
}

run_bench_geth_calls(){
    run_bench traces_geth_calls "geth_calls -b 20033436:20033500"
}

main(){
    # Parse named arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --benches-id)
                BENCHES_ID="$2"
                shift 2
                ;;
            --output-base-path)
                OUTPUT_BASE_PATH="$2"
                shift 2
                ;;
            *)
                echo "Unknown parameter passed: $1"
                exit 1
                ;;
        esac
    done

    run_bench_blocks_to_tip
    run_bench_blocks_range
    run_bench_tx_range1
    run_bench_tx_range2
    run_bench_logs_range1
    run_bench_logs_range2
    run_bench_state_reads
    run_bench_geth_calls
}

main "${@}"
