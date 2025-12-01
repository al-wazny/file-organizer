use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long, default_value = ".")]
    path: String,
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args.path);
}
