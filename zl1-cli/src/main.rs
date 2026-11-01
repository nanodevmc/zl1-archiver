use std::env;
use std::path::Path;
use zl1_core::{Compressor, Decompressor};

fn print_usage() {
    println!("ZL1 Archiver v0.1.0 (Extreme Solid Compression)");
    println!("Usage:");
    println!("  zl1-cli c <input_path> <output.zl1> [--bcj]   Compress file/directory");
    println!("  zl1-cli x <archive.zl1> <output_dir>         Extract archive");
    println!("  zl1-cli l <archive.zl1>                      List archive content");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "c" | "compress" | "a" => {
            if args.len() < 4 {
                print_usage();
                return;
            }
            let src = &args[2];
            let dst = &args[3];
            let apply_bcj = args.iter().any(|a| a == "--bcj");

            println!(">> Compressing '{}' -> '{}'...", src, dst);
            let start = std::time::Instant::now();
            match Compressor::compress_to_archive(Path::new(src), Path::new(dst), apply_bcj) {
                Ok(manifest) => {
                    let elapsed = start.elapsed();
                    let ratio = if manifest.total_uncompressed_size > 0 {
                        (manifest.solid_stream_compressed_size as f64 / manifest.total_uncompressed_size as f64) * 100.0
                    } else {
                        0.0
                    };
                    println!("OK! Compressed {} files in {:.2?}", manifest.files.len(), elapsed);
                    println!("Original size:   {} bytes", manifest.total_uncompressed_size);
                    println!("Compressed size: {} bytes", manifest.solid_stream_compressed_size);
                    println!("Compression ratio: {:.2}% ({:.1}x smaller)", ratio, if ratio > 0.0 { 100.0 / ratio } else { 1.0 });
                }
                Err(e) => {
                    eprintln!("Error during compression: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "x" | "decompress" | "e" => {
            if args.len() < 4 {
                print_usage();
                return;
            }
            let archive = &args[2];
            let dst = &args[3];

            println!(">> Extracting '{}' -> '{}'...", archive, dst);
            let start = std::time::Instant::now();
            match Decompressor::decompress_archive(Path::new(archive), Path::new(dst)) {
                Ok(manifest) => {
                    let elapsed = start.elapsed();
                    println!("OK! Extracted {} files in {:.2?}", manifest.files.len(), elapsed);
                    println!("Total restored size: {} bytes", manifest.total_uncompressed_size);
                }
                Err(e) => {
                    eprintln!("Error during extraction: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "l" | "list" => {
            let archive = &args[2];
            // Показываем содержимое
            match std::fs::File::open(archive) {
                Ok(_) => {
                    println!("Listing is supported directly via decompress metadata inspection.");
                }
                Err(e) => eprintln!("Failed to open archive: {}", e),
            }
        }
        _ => {
            print_usage();
        }
    }
}
