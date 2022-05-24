use anyhow::Result;
use query_handler::{contributor_query_handler, repository_query_handler};

mod query_handler;
mod report_generator;

#[derive(Clone, Debug)]
pub struct BusFactorQueryCommand {
    pub language: String,
    pub project_count: u32,
    pub github_url: String,
}
pub fn initialize(command: BusFactorQueryCommand) -> Result<()> {
    validatate_command(&command);
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        report_generator::ReportGenerator::run(
            contributor_query_handler::ContributorHandler::run(
                repository_query_handler::RepositoryHandler::run(command).await?,
            )
            .await?,
        )
        .await
    })?;
    log::debug!("Finalising");
    Ok(())
}

fn validatate_command(command: &BusFactorQueryCommand) {
    if command.project_count == 0 {
        std::process::exit(0);
    }
    if command.language.is_empty() {
        std::process::exit(0);
    }
}
