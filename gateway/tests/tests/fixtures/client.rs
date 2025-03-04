use std::path::PathBuf;

use async_trait::async_trait;
use bitcoin::{secp256k1, KeyPair};
use fedimint_api::config::ClientConfig;
use ln_gateway::{
    client::{DbFactory, IGatewayClientBuilder},
    LnGatewayError,
};
use mint_client::{
    api::{FederationApi, WsFederationConnect},
    Client, FederationId, GatewayClient, GatewayClientConfig,
};
use secp256k1::{PublicKey, Secp256k1};
use url::Url;

use super::fed::MockApi;

#[derive(Debug, Clone)]
pub struct TestGatewayClientBuilder {
    db_factory: DbFactory,
}

impl TestGatewayClientBuilder {
    pub fn new(db_factory: DbFactory) -> Self {
        Self { db_factory }
    }
}

#[async_trait]
impl IGatewayClientBuilder for TestGatewayClientBuilder {
    async fn build(
        &self,
        config: GatewayClientConfig,
    ) -> Result<Client<GatewayClientConfig>, LnGatewayError> {
        let federation_id = FederationId(config.client_config.federation_name.clone());

        let api: FederationApi = MockApi::new().into();
        let db = self
            .db_factory
            .create_database(federation_id, PathBuf::new())?;

        Ok(GatewayClient::new_with_api(config, db, api, Default::default()).await)
    }

    async fn create_config(
        &self,
        _connect: WsFederationConnect,
        node_pubkey: PublicKey,
        announce_address: Url,
    ) -> Result<GatewayClientConfig, LnGatewayError> {
        // TODO: use the connect info urls to get the federation name?
        // Simulate clients in the same federation by seeding the generated `client_config`
        // Using some of the info in provided web socket connect info
        let client_config = ClientConfig {
            federation_name: "".to_string(),
            epoch_pk: threshold_crypto::SecretKey::random().public_key(),
            nodes: [].into(),
            modules: [].into(),
        };

        let mut rng = rand::rngs::OsRng;
        let ctx = Secp256k1::new();
        let kp_fed = KeyPair::new(&ctx, &mut rng);

        Ok(GatewayClientConfig {
            client_config,
            redeem_key: kp_fed,
            timelock_delta: 10,
            node_pub_key: node_pubkey,
            api: announce_address,
        })
    }

    fn save_config(&self, _config: GatewayClientConfig) -> Result<(), LnGatewayError> {
        // noop: don't save configs
        Ok(())
    }

    fn load_configs(&self) -> Result<Vec<GatewayClientConfig>, LnGatewayError> {
        // noop: return empty config list
        Ok([].into())
    }
}
