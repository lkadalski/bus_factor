use std::sync::Arc;

use crate::query_handler::{HttpClientDetails, Other, RepositoryQueryResult};
use crate::BusFactorQueryCommand;
use anyhow::{anyhow, Context, Result};
use log;
use reqwest::Url;
use serde::Deserialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};

const PAGE_SIZE: u32 = 30;

pub(crate) struct RepositoryHandler;

impl RepositoryHandler {
    pub(crate) async fn run(
        command: BusFactorQueryCommand,
    ) -> Result<Receiver<RepositoryQueryResult>> {
        let (tx, rx) = channel(1000);
        tokio::spawn(RepositoryHandler::fetch_data(command, tx));
        log::trace!("RepositoryHandler initialised");
        Ok(rx)
    }
    async fn fetch_data(command: BusFactorQueryCommand, tx: Sender<RepositoryQueryResult>) {
        if let Err(err) = RepositoryHandler::query_api(command, tx).await {
            eprintln!("Error during fetching repository API: {err}");
            std::process::exit(1);
        }
    }
    async fn query_api(
        command: BusFactorQueryCommand,
        tx: Sender<RepositoryQueryResult>,
    ) -> Result<()> {
        let client_details = Arc::new(HttpClientDetails::new(command.clone())?);

        let page_size = Self::determine_page_size(&command);

        let initial_response_count = Self::fetch_page_of_results(
            create_repo_url(1, client_details.clone(), page_size)?,
            client_details.clone(),
            tx.clone(),
        )
        .await?;

        let page_request_count = Self::determine_page_count(command.project_count, page_size);

        log::info!(
            "Total items to fetch = {}, pages = {}",
            initial_response_count,
            page_request_count
        );

        if initial_response_count > (command.project_count - page_size) {
            for request_no in 2..=page_request_count {
                let full_url = create_repo_url(request_no, client_details.clone(), page_size)?;
                tokio::spawn(Self::fetch_page_of_results(
                    full_url,
                    client_details.clone(),
                    tx.clone(),
                ));
            }
        }

        Ok(())
    }
    fn determine_page_count(project_count: u32, page_size: u32) -> u32 {
        let mut page_request_count = project_count / page_size;
        if project_count % page_size > 0 {
            page_request_count = page_request_count + 1;
        }
        page_request_count
    }

    fn determine_page_size(command: &BusFactorQueryCommand) -> u32 {
        if PAGE_SIZE > command.project_count {
            command.project_count
        } else {
            PAGE_SIZE
        }
    }
    async fn fetch_page_of_results(
        full_url: Url,
        client_details: Arc<HttpClientDetails>,
        tx: Sender<RepositoryQueryResult>,
    ) -> Result<u32> {
        log::trace!("Targeting {:?}", &full_url);
        let request = client_details.client.get(full_url);
        log::trace!("{:?}", &request);
        let response = request.send().await?.json::<RepositoryResponse>().await?;
        log::trace!("{:?}", &response);
        let total_count = response.total_count;
        tokio::spawn(async move {
            for project in response.items {
                tx.send(RepositoryQueryResult {
                    stargazers: project.stargazers_count,
                    contributor_url: project.contributors_url,
                    project_name: project.full_name,
                    client_details: client_details.clone(),
                })
                .await
                .expect("Could not send message to contributor query handler");
            }
        });
        Ok(total_count)
    }
}
fn create_repo_url(
    request_no: u32,
    client_details: Arc<HttpClientDetails>,
    page_size: u32,
) -> Result<Url> {
    reqwest::Url::parse_with_params(
        &format!(
            "{}?q=language:{}",
            client_details.command.github_url, client_details.command.language
        ),
        &[
            ("sort", "stars"),
            ("order", "desc"),
            ("page", &request_no.to_string()),
            ("per_page", &page_size.to_string()),
        ],
    )
    .map_err(|err| anyhow!(err))
    .context("Could not create URL")
}

#[derive(Deserialize, Debug)]
pub struct RepositoryResponse {
    total_count: u32,
    items: Vec<RepositoryDetails>,
    #[serde(flatten, skip)]
    other: Other,
}
#[derive(Deserialize, Debug)]
struct RepositoryDetails {
    stargazers_count: u32,
    contributors_url: String,
    full_name: String,
    #[serde(flatten, skip)]
    other: Other,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculate_page_size_for_typical_use_cases_beside_first_page() {
        let result = RepositoryHandler::determine_page_count(40, 30);
        assert_eq!(result, 2);
        let result = RepositoryHandler::determine_page_count(60, 30);
        assert_eq!(result, 2);
        let result = RepositoryHandler::determine_page_count(99, 30);
        assert_eq!(result, 4);
    }
}
