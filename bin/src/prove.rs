use clap::Parser;
use log::info;
use prover::utils::{get_block_trace_from_file, init_env_and_log, load_or_create_params};
use prover::zkevm::{
    circuit::{SuperCircuit, AGG_DEGREE},
    Prover,
};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Get params and write into file.
    #[clap(short, long = "params")]
    params_path: Option<String>,
    /// Get BlockTrace from file or dir.
    #[clap(short, long = "trace")]
    trace_path: Option<String>,
    /// Option means if generates super circuit proof.
    /// Boolean means if output super circuit proof.
    #[clap(long = "super")]
    super_proof: Option<bool>,
    /// Option means if generates compressed chunk proof.
    /// Boolean means if output chunk proof.
    #[clap(long = "chunk")]
    chunk_proof: Option<bool>,
}

fn main() {
    init_env_and_log("prove");

    let args = Args::parse();
    let agg_params = load_or_create_params(&args.params_path.unwrap(), *AGG_DEGREE)
        .expect("failed to load or create params");

    let mut prover = Prover::from_params(agg_params);

    let mut traces = HashMap::new();
    let trace_path = PathBuf::from(&args.trace_path.unwrap());
    if trace_path.is_dir() {
        for entry in fs::read_dir(trace_path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() && path.to_str().unwrap().ends_with(".json") {
                let block_trace = get_block_trace_from_file(path.to_str().unwrap());
                traces.insert(path.file_stem().unwrap().to_os_string(), block_trace);
            }
        }
    } else {
        let block_trace = get_block_trace_from_file(trace_path.to_str().unwrap());
        traces.insert(trace_path.file_stem().unwrap().to_os_string(), block_trace);
    }

    let outer_now = Instant::now();
    for (trace_name, trace) in traces {
        if args.super_proof.is_some() {
            let proof_path = PathBuf::from(&trace_name).join("super.proof");
            let block_hash = trace.header.hash.unwrap();

            let now = Instant::now();
            let super_proof = prover
                .gen_inner_proof::<SuperCircuit>(&[trace.clone()])
                .expect("cannot generate super circuit proof");
            info!(
                "finish generating super circuit proof for block {}, elapsed: {:?}",
                block_hash,
                now.elapsed()
            );

            if args.super_proof.unwrap() {
                let mut f = File::create(&proof_path).unwrap();
                f.write_all(super_proof.proof.as_slice()).unwrap();
            }
        }

        if args.chunk_proof.is_some() {
            let mut proof_path = PathBuf::from(&trace_name).join("agg.proof");
            let block_hash = trace.header.hash.unwrap();

            let agg_pk_path = PathBuf::from(&trace_name).join("agg.pk");

            let now = Instant::now();
            let chunk_proof = prover
                .gen_chunk_proof(&[trace], Some(&agg_pk_path))
                .expect("cannot generate chunk proof");
            info!(
                "finish generating chunk proof for block {}, elapsed: {:?}",
                block_hash,
                now.elapsed()
            );

            if args.chunk_proof.unwrap() {
                fs::create_dir_all(&proof_path).unwrap();
                chunk_proof.dump(&mut proof_path, "chunk").unwrap();
            }
        }
    }
    info!("finish generating all, elapsed: {:?}", outer_now.elapsed());
}
