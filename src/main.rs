mod message_broker;
mod message_bus;
mod rabbitmq_bus;

use message_broker::MessageBroker;
use message_bus::{MessageBrokerExchanges, MessageBrokerQueues, MessageBrokerRoutingKeys};

#[tokio::main]
async fn main() {
    let broker = MessageBroker::new().await.unwrap();

    broker
        .listen(
            &MessageBrokerQueues::TestQueue,
            &MessageBrokerExchanges::TestExchange,
            &MessageBrokerRoutingKeys::all(),
        )
        .await
        .unwrap();

    std::future::pending::<()>().await;
}
