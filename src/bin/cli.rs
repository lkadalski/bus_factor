use anyhow::Result;
use bus_factor::BusFactorQueryCommand;
use clap::Parser;

/// Simple program to fetch GitHub's projects which have bus factor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Name of the programming language
    #[clap(short, long)]
    language: String,
    //Number of projects to consider
    #[clap(short, long, default_value = "10")]
    project_count: u32,
    #[clap(
        short,
        long,
        default_value = "https://api.github.com/search/repositories"
    )]
    github_url: String,
}

impl From<Args> for BusFactorQueryCommand {
    fn from(cli_args: Args) -> Self {
        BusFactorQueryCommand {
            language: cli_args.language,
            project_count: cli_args.project_count,
            github_url: cli_args.github_url,
        }
    }
}
fn main() -> Result<()> {
    env_logger::init();
    log::debug!("Starting Bus Factor");
    let args = Args::parse();

    let bus_factor_arguments: BusFactorQueryCommand = args.into();
    log::debug!(
        "Fetching data about top {} {} projects with highest stargazers number",
        &bus_factor_arguments.project_count,
        &bus_factor_arguments.language
    );
    bus_factor::initialize(bus_factor_arguments)
}
