use anyhow::Result;
use query_handler::{bus_factor, repository};

mod query_handler;
mod report_generator;

#[derive(Clone, Debug)]
pub struct BusFactorQueryCommand {
    pub language: String,
    pub project_count: u32,
    pub github_url: String,
}
///Utilize Async Pipeline Design Pattern
///# Errors
///May fail when can not create Runtime/Channels.
pub fn initialize(command: BusFactorQueryCommand) -> Result<()> {
    validatate_command(&command);
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        report_generator::ReportGenerator::run(
            bus_factor::ContributorHandler::run(repository::RepositoryHandler::run(command).await?)
                .await?,
        )
        .await
    })?;
    log::debug!("Finalising");
    Ok(())
}
//Immediate results
fn validatate_command(command: &BusFactorQueryCommand) {
    if command.project_count == 0 {
        std::process::exit(0);
    }
    if command.language.is_empty() {
        std::process::exit(0);
    }
}
