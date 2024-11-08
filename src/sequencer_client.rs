use astria_sequencer_client::*;
use color_eyre::eyre::{self, eyre, WrapErr as _};
pub(super) struct SequencerClient {
    pub sequencer_client: HttpClient,
}

impl SequencerClient {
    pub(crate) fn new(sequencer_url: &str) -> eyre::Result<Self> {
        let client =
            HttpClient::new(sequencer_url).wrap_err("failed connecting to sequencer rpc")?;
        Ok(SequencerClient {
            sequencer_client: client,
        })
    }
    pub(crate) async fn get_nonce(&self, address: Address) -> eyre::Result<NonceResponse> {
        self.sequencer_client
            .get_latest_nonce(address)
            .await
            .wrap_err(format!(
                "failed getting latest nonce for address {}",
                address
            ))
    }
    pub(crate) async fn get_balance(&self, address: Address) -> eyre::Result<BalanceResponse> {
        self.sequencer_client
            .get_latest_balance(address)
            .await
            .wrap_err(format!("failed getting balance for address {}", address))
    }
}
