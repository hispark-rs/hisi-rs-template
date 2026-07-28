use hisi_rf::ws63::{SelectedProfile, Storage};
use std::{env, fs, process};

const EVENT_CAPACITY: usize = 8;

fn main() {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(output_path) = args.next() else {
        eprintln!("usage: rf-resource-report <output.json>");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("rf-resource-report accepts exactly one output path");
        process::exit(2);
    }

    let storage = Storage::<SelectedProfile, EVENT_CAPACITY>::new();
    let mut output = String::new();
    storage
        .report()
        .write_json(&mut output)
        .expect("String writes are infallible");
    output.push('\n');
    if let Err(error) = fs::write(&output_path, output) {
        eprintln!(
            "failed to write RF resource report to {}: {error}",
            output_path.to_string_lossy()
        );
        process::exit(1);
    }
}
