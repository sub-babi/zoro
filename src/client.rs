use bazuka::client::NodeError;
use std::future::Future;

#[derive(Clone)]
pub struct SyncClient {
    node: bazuka::client::PeerAddress,
    network: String,
    miner_token: String,
    sk: <bazuka::core::Signer as bazuka::crypto::SignatureScheme>::Priv,
}

impl SyncClient {
    pub fn new(node: bazuka::client::PeerAddress, network: &str, miner_token: String) -> Self {
        Self {
            node,
            network: network.to_string(),
            miner_token,
            sk: <bazuka::core::Signer as bazuka::crypto::SignatureScheme>::generate_keys(b"dummy")
                .1,
        }
    }
    async fn call<
        R,
        Fut: Future<Output = Result<R, NodeError>>,
        F: FnOnce(bazuka::client::BazukaClient) -> Fut,
    >(
        &self,
        f: F,
    ) -> Result<R, NodeError> {
        let (lp, client) = bazuka::client::BazukaClient::connect(
            self.sk.clone(),
            self.node,
            self.network.clone(),
            Some(self.miner_token.clone()),
        );

        let (res, _) = tokio::join!(
            async move { Ok::<_, bazuka::client::NodeError>(f(client).await) },
            lp
        );
        Ok(res??)
    }
    pub async fn is_outdated(&self) -> Result<bool, NodeError> {
        self.call(move |client| async move {
            Ok(!client.outdated_heights().await?.outdated_heights.is_empty())
        })
        .await
    }
    pub async fn transact(
        &self,
        tx: bazuka::core::TransactionAndDelta,
    ) -> Result<bazuka::client::messages::TransactResponse, NodeError> {
        self.call(move |client| async move { Ok(client.transact(tx).await?) })
            .await
    }
    pub async fn get_account(
        &self,
        address: bazuka::core::Address,
    ) -> Result<bazuka::client::messages::GetAccountResponse, NodeError> {
        self.call(move |client| async move { Ok(client.get_account(address).await?) })
            .await
    }
    pub async fn get_zero_mempool(
        &self,
    ) -> Result<bazuka::client::messages::GetZeroMempoolResponse, NodeError> {
        self.call(move |client| async move { Ok(client.get_zero_mempool().await?) })
            .await
    }
    pub async fn is_mining(&self) -> Result<bool, NodeError> {
        self.call(move |client| async move {
            Ok(client
                .get_miner_puzzle()
                .await
                .map(|resp| resp.puzzle.is_some())?)
        })
        .await
    }
    pub async fn get_height(&self) -> Result<u64, NodeError> {
        self.call(move |client| async move { Ok(client.stats().await.map(|resp| resp.height)?) })
            .await
    }
}
