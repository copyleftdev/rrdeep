use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::exit;
use colored::*;
use chrono::{DateTime, Utc};
use crate::rrdeep::{compare_rrdeep, compute_rrdeep_from_path_concurrent};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cmd {
    #[command(subcommand)]
    cmd: Sub
}

#[derive(Subcommand)]
enum Sub {
    CompareFiles { file1: PathBuf, file2: PathBuf },
    Compare { sig1: String, sig2: String },
}

pub fn run() {
    let cmd = Cmd::parse();
    match cmd.cmd {
        Sub::CompareFiles { file1, file2 } => {
            compare_files(file1, file2);
        }
        Sub::Compare { sig1, sig2 } => {
            compare_signatures(sig1, sig2);
        }
    }
}

fn compare_files(file1: PathBuf, file2: PathBuf) {
    let meta1 = match fs::metadata(&file1) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: Could not read metadata for {}: {}", file1.display(), e);
            exit(1);
        }
    };
    let meta2 = match fs::metadata(&file2) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: Could not read metadata for {}: {}", file2.display(), e);
            exit(1);
        }
    };

    let size1 = meta1.len();
    let size2 = meta2.len();
    let mt1 = meta1.modified().unwrap_or(UNIX_EPOCH);
    let mt2 = meta2.modified().unwrap_or(UNIX_EPOCH);

    // Now we get both the signature & performance metrics
    let (sig1, perf1) = match compute_rrdeep_from_path_concurrent(&file1) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: Could not read file {}: {}", file1.display(), e);
            exit(1);
        }
    };
    let (sig2, perf2) = match compute_rrdeep_from_path_concurrent(&file2) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: Could not read file {}: {}", file2.display(), e);
            exit(1);
        }
    };

    let score = compare_rrdeep(&sig1, &sig2);
    let res = match score {
        s if s >= 80 => "Very Similar".green().bold(),
        s if s >= 50 => "Similar".yellow().bold(),
        _ => "Different".red().bold(),
    };

    println!();
    println!("{}", "[RRDeep] Comparing:".bold());
    println!("  {}", file1.display());
    println!("    - Size: {} bytes", size1);
    println!("    - Modified: {}", format_time(mt1));
    println!();
    println!("  {}", file2.display());
    println!("    - Size: {} bytes", size2);
    println!("    - Modified: {}", format_time(mt2));
    println!();
    println!("{}", "Signatures:".bold());
    println!("  {} -> {}", file1.display(), sig1);
    println!("  {} -> {}", file2.display(), sig2);
    println!();
    println!("{} {}", "Similarity Score:".bold(), score);
    println!("{} {}", "Result:".bold(), res);

    // Print performance metrics for each file
    println!();
    println!("Performance Metrics:");
    println!(
        "  {} => processed {} bytes in {:.3}s => {:.2} MB/s",
        file1.display(),
        perf1.total_bytes,
        perf1.duration_s,
        perf1.speed_mbps
    );
    println!(
        "  {} => processed {} bytes in {:.3}s => {:.2} MB/s",
        file2.display(),
        perf2.total_bytes,
        perf2.duration_s,
        perf2.speed_mbps
    );
    println!();
}

fn compare_signatures(sig1: String, sig2: String) {
    let score = compare_rrdeep(&sig1, &sig2);

    let res = match score {
        s if s >= 80 => "Very Similar".green().bold(),
        s if s >= 50 => "Similar".yellow().bold(),
        _ => "Different".red().bold(),
    };

    println!();
    println!("{}", "[RRDeep] Comparing Signatures:".bold());
    println!("  sig1: {}", sig1);
    println!("  sig2: {}", sig2);
    println!();
    println!("{} {}", "Similarity Score:".bold(), score);
    println!("{} {}", "Result:".bold(), res);
    println!();
}

fn format_time(t: SystemTime) -> String {
    let dt: DateTime<Utc> = t.into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}
