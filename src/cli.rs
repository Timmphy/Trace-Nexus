use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "trace-nexus")]
#[command(author = "Tphy")]
#[command(version = "0.1.0")]
#[command(about = "Lightweight forensic artifact collector", long_about = None)]
pub struct Cli {
    #[arg(long, conflicts_with = "full")]
    pub light: bool,

    #[arg(long, conflicts_with = "light")]
    pub full: bool,
}
