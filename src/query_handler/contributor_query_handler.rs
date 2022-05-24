use crate::query_handler::{Other, RepositoryBusFactorResult, RepositoryQueryResult};
use anyhow::{anyhow, Context, Result};
use reqwest::Url;
use serde::Deserialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub(crate) struct ContributorHandler;
impl ContributorHandler {
    pub(crate) async fn run(
        mut receiver: Receiver<RepositoryQueryResult>,
    ) -> Result<Receiver<RepositoryBusFactorResult>> {
        let (tx, rx) = channel(1000);
        tokio::task::spawn(async move {
            while let Some(data) = receiver.recv().await {
                tokio::spawn(Self::fetch_data(data, tx.clone()));
            }
        });
        log::debug!("ContributorHandler initialised");
        Ok(rx)
    }

    async fn fetch_data(data: RepositoryQueryResult, tx: Sender<RepositoryBusFactorResult>) {
        if let Err(err) = Self::query_api(data, tx).await {
            eprintln!("Error during fetching contributors API: {err}");
            std::process::exit(1);
        }
    }
    async fn query_api(
        data: RepositoryQueryResult,
        tx: Sender<RepositoryBusFactorResult>,
    ) -> Result<()> {
        let response = Self::fetch_page_of_results(&data).await?;
        let bus_factor_detected =
            Self::detect_bus_factor(&response.contributors, data.project_name, data.stargazers);
        if let Some(bus_factor) = bus_factor_detected {
            tx.send(bus_factor).await?;
        }
        Ok(())
    }
    fn detect_bus_factor(
        contributors: &[ContributorDetails],
        project_name: String,
        star_gazers: u32,
    ) -> Option<RepositoryBusFactorResult> {
        log::trace!("Calculating bus factor for {project_name}");
        let contributors_total_commits = Self::calculate_commits_sum(&contributors);
        contributors.iter().find_map(|contributor| {
            let percentage = (100 * contributor.contributions) / contributors_total_commits;
            log::trace!("Calculated percentage is {percentage:?} for project {project_name}");
            if percentage >= 75 {
                log::info!("Project {project_name} has a busfactor");
                return Some(RepositoryBusFactorResult {
                    login: contributor.login.clone(),
                    contributions: contributor.contributions,
                    repo_name: project_name.clone(),
                    bus_factor: percentage,
                    stargazers: star_gazers,
                });
            }
            None
        })
    }

    fn calculate_commits_sum(contributors: &[ContributorDetails]) -> u32 {
        contributors
            .iter()
            .fold(0, |prev, next| prev + next.contributions)
    }

    fn create_contrib_url(base_url: String) -> Result<Url> {
        Url::parse_with_params(
            &base_url,
            &[
                ("sort", "contributions"),
                ("order", "desc"),
                ("per_page", "25"),
            ],
        )
        .map_err(|err| anyhow!(err))
        .with_context(|| "Could not create URL")
    }

    async fn fetch_page_of_results(
        query_result: &RepositoryQueryResult,
    ) -> Result<ContributorsResponse> {
        let full_url = Self::create_contrib_url(query_result.contributor_url.clone())?;
        log::trace!("Targeting {:?}", &full_url);
        let request = query_result.client_details.client.get(full_url);
        log::trace!("{:?}", &request);
        let response = request.send().await?.json::<ContributorsResponse>().await?;
        log::trace!("{:?}", &response);
        Ok(response)
    }
}
#[derive(Deserialize, Debug)]
#[serde(transparent)]
struct ContributorsResponse {
    contributors: Vec<ContributorDetails>,
}
#[derive(Deserialize, Debug)]
struct ContributorDetails {
    login: String,
    contributions: u32,
    #[serde(flatten)]
    other: Other,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calculate_bus_factor_simple() {
        let result = ContributorHandler::detect_bus_factor(
            &[ContributorDetails {
                login: "luke".to_string(),
                contributions: 1000,
                other: Default::default(),
            }],
            "minigun".to_string(),
            100,
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().bus_factor, 100)
    }
    #[test]
    fn calculate_bus_factor_advance() {
        let result = ContributorHandler::detect_bus_factor(
            &[
                ContributorDetails {
                    login: "luke".to_string(),
                    contributions: 1000,
                    other: Default::default(),
                },
                ContributorDetails {
                    login: "kubot".to_string(),
                    contributions: 500,
                    other: Default::default(),
                },
            ],
            "minigun".to_string(),
            100,
        );
        assert!(result.is_none());
    }
    #[test]
    fn calculate_bus_factor_almost() {
        let result = ContributorHandler::detect_bus_factor(
            &[
                ContributorDetails {
                    login: "luke".to_string(),
                    contributions: 750,
                    other: Default::default(),
                },
                ContributorDetails {
                    login: "kubot".to_string(),
                    contributions: 250,
                    other: Default::default(),
                },
            ],
            "minigun".to_string(),
            100,
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().bus_factor, 75)
    }
}
