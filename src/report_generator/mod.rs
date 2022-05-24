use crate::query_handler::RepositoryBusFactorResult;
use anyhow::Result;
use itertools::Itertools;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

pub(crate) struct ReportGenerator {}
type DataStorage = Arc<Mutex<Vec<RepositoryBusFactorResult>>>;
impl ReportGenerator {
    pub(crate) async fn run(receiver: Receiver<RepositoryBusFactorResult>) -> Result<()> {
        tokio::task::spawn(Self::process_results(receiver)).await??;
        log::trace!("Closing Report Generator");
        Ok(())
    }
    async fn process_results(mut receiver: Receiver<RepositoryBusFactorResult>) -> Result<()> {
        log::trace!("About to start consuming");
        let final_data: DataStorage = Arc::new(Mutex::new(vec![]));
        while let Some(data) = receiver.recv().await {
            log::info!("we have data {data:?}");
            Self::save_data(data, final_data.clone()).await;
        }
        log::trace!("Closing Report Generator channel");
        Self::print_data(final_data).await;
        Ok(())
    }
    async fn save_data(data: RepositoryBusFactorResult, data_storage: DataStorage) {
        let mut lock = data_storage.lock().await;
        lock.push(data);
    }
    async fn print_data(data_storage: Arc<Mutex<Vec<RepositoryBusFactorResult>>>) {
        let data = data_storage.lock().await;
        data.iter()
            .sorted_unstable_by_key(|bus_factor| bus_factor.bus_factor)
            .rev()
            .for_each(|print| {
                println!(
                    "project: {:20}\t\tuser: {:20}\t\tpercentage: {}",
                    print.repo_name, print.login, print.bus_factor
                )
            });
    }
}
