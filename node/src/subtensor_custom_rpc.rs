use std::sync::Arc;
use futures::StreamExt;
use jsonrpsee::{RpcModule, SubscriptionSink};
use jsonrpsee::types::{RpcParams, ConnectionId};
use sc_transaction_pool_api::{TransactionPool, ImportNotificationStream};
use sp_runtime::codec::Encode;
use jsonrpsee::tokio;
use serde_json::json;

pub struct SubtensorCustomRpc<T> {
    pool: Arc<T>,
}

impl<T> SubtensorCustomRpc<T>
where
    T: TransactionPool + 'static,
{
    pub fn new(pool: Arc<T>) -> Self {
        Self { pool }
    }

    pub fn into_rpc(self) -> RpcModule<()> {
        let mut module = RpcModule::new(());

        module
            .register_subscription(
                "author_subscribeExtrinsics",
                "author_unsubscribeExtrinsics",
                "extrinsics_subscribe",
                move |_params: RpcParams, sink: SubscriptionSink, _conn: ConnectionId, _method| {
                    let mut stream: ImportNotificationStream<T::Hash> = self.pool.import_notification_stream();

                    tokio::spawn(async move {
                        while let Some(notification) = stream.next().await {
                            let encoded_notification = hex::encode(notification.encode());
                            let message = serde_json::json!({ "extrinsic": encoded_notification });

                            if sink.send(message).await.is_err() {
                                break;
                            }
                        }
                    });

                    Ok(())
                },
            )
            .expect("Subscription must register successfully.");

        module
    }
}
