//! fluidc: Fluid Rust Compiler CLI

use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line
    if args.len() < 2 {
        print_help();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "--version" => {
            println!("fluidc 0.1.0");
            process::exit(0);
        }
        "--help" => {
            print_help();
            process::exit(0);
        }
        "compile" => {
            if args.len() < 3 {
                eprintln!("Error: compile requires a source file");
                print_help();
                process::exit(1);
            }
            cmd_compile(&args[2..]);
        }
        "emit-rmir" => {
            if args.len() < 3 {
                eprintln!("Error: emit-rmir requires a source file");
                process::exit(1);
            }
            cmd_emit_rmir(&args[2..]);
        }
        "prove" => {
            if args.len() < 3 {
                eprintln!("Error: prove requires a source file");
                process::exit(1);
            }
            cmd_prove(&args[2..]);
        }
        _ => {
            eprintln!("Error: unknown command '{}'", command);
            print_help();
            process::exit(1);
        }
    }
}

fn cmd_compile(args: &[String]) {
    let source_file = &args[0];
    let mut backend = "native";
    let mut output = None;

    // Parse options
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--backend" => {
                if i + 1 < args.len() {
                    backend = &args[i + 1];
                    i += 2;
                } else {
                    eprintln!("Error: --backend requires an argument");
                    process::exit(1);
                }
            }
            "-o" => {
                if i + 1 < args.len() {
                    output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    process::exit(1);
                }
            }
            _ => {
                eprintln!("Error: unknown option '{}'", args[i]);
                process::exit(1);
            }
        }
    }

    let path = Path::new(source_file);

    match fluid_rust_compiler::compile(path, backend, "generate") {
        Ok(artifact) => {
            if artifact.has_errors() {
                for diag in &artifact.diagnostics {
                    eprintln!("error: {}", diag.message);
                }
                process::exit(1);
            }

            println!(
                "✓ Compiled: {} ({} bytes RMIR)",
                source_file,
                artifact.rmir_bytecode.len()
            );

            if let Some(output_path) = output {
                match std::fs::write(&output_path, &artifact.rmir_bytecode) {
                    Ok(_) => println!("✓ Wrote RMIR to: {}", output_path),
                    Err(e) => {
                        eprintln!("Error writing output: {}", e);
                        process::exit(1);
                    }
                }
            }

            println!("RMIR hash: {}", artifact.rmir_hash);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_emit_rmir(args: &[String]) {
    let source_file = &args[0];
    let mut output = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 < args.len() {
                    output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    process::exit(1);
                }
            }
            _ => i += 1,
        }
    }

    let path = Path::new(source_file);

    match fluid_rust_compiler::compile(path, "proof-only", "generate") {
        Ok(artifact) => {
            println!("✓ RMIR emitted: {} bytes", artifact.rmir_bytecode.len());
            println!("Hash: {}", artifact.rmir_hash);

            if let Some(output_path) = output {
                match std::fs::write(&output_path, &artifact.rmir_bytecode) {
                    Ok(_) => println!("✓ Wrote to: {}", output_path),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_prove(args: &[String]) {
    let source_file = &args[0];
    let path = Path::new(source_file);

    match fluid_rust_compiler::compile(path, "proof-only", "prove") {
        Ok(artifact) => {
            println!("✓ Proof mode: generating proof obligations");
            println!("RMIR hash: {}", artifact.rmir_hash);
            println!("Proof obligations: {} bytes", artifact.rmir_bytecode.len());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("FLUID RUST COMPILER v0.1.0");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    fluidc <COMMAND> [OPTIONS] <SOURCE>");
    eprintln!();
    eprintln!("COMMANDS:");
    eprintln!("    compile         Compile to RMIR (default: native backend)");
    eprintln!("    emit-rmir       Emit only RMIR bytecode");
    eprintln!("    prove           Generate proof obligations only");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    --backend <BACKEND>   Target backend: native, managed, wasm (default: native)");
    eprintln!("    -o <OUTPUT>           Write output to file");
    eprintln!("    --version             Print version and exit");
    eprintln!("    --help                Print this message");
}
