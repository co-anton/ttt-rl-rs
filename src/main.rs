use clap::Parser;

mod agent;
mod gui;
mod logic;

#[derive(Parser, Debug)]
#[command(
    author = "Antonin Couturier",
    version = "0.1",
    about = "Tic-Tac-Toe with Reinforcement Learning Agent"
)]
struct Args {
    #[clap(long)]
    gui: bool,

    #[clap(long)]
    training: bool,

    #[clap(long)]
    evaluation: bool,
}

fn main() {
    let args = Args::parse();
    if args.gui {
        gui::TicTacToeApp::run();
    } else if args.training {
        agent::train(1000, 100);
    }
}
