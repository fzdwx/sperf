use clap::Parser;

fn main() {
    Cli::parse().run();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command name or command alias or command path
    command: String,
    /// command arguments
    args: Option<Vec<String>>,
}

impl Cli {
    fn run(&self) {
        println!("Command: {}", self.command);
        println!("Args: {:?}", self.args);
    }
}
