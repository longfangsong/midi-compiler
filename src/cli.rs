use clap::Parser;
use midi_compiler_lib::convert_midi;
use std::fs;

/// Compile midi file to simple (frequency, time) binary format
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();
    let bytes_in = fs::read(&args.input).unwrap();
    let result = convert_midi(bytes_in);
    fs::write(&args.output, result).unwrap();
}
