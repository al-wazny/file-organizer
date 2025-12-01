use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long, default_value = ".")]
    path: String,

    #[clap(short, long, num_args(0..))]
    extensions: Vec<String>,
    
    #[clap(short, long)]
    dry_run: bool,
    
    #[clap(short, long)]
    backup: bool,
}

fn main() {
    let args = Cli::parse();
    println!("{:#?}", args);
}
