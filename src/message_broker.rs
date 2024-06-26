use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use crate::message_bus::{
    MessageBrokerExchanges, MessageBrokerQueues, MessageBrokerRoutingKeys, MessageBus,
};
use crate::rabbitmq_bus::RabbitMQBus;

pub struct MessageBroker {
    bus: Box<dyn for<'a> MessageBus<'a> + Send + Sync>,
}

impl MessageBroker {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let bus = Box::new(RabbitMQBus::new().await?);
        Ok(Self { bus })
    }

    pub fn publish<'a>(
        &'a self,
        exchange: &'a MessageBrokerExchanges,
        routing_key: &'a MessageBrokerRoutingKeys,
        message: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>> {
        self.bus
            .publish(exchange.as_str(), routing_key.as_str(), message)
    }

    pub fn listen<'a>(
        &'a self,
        queue: &'a MessageBrokerQueues,
        exchange: &'a MessageBrokerExchanges,
        routing_keys: &'a Vec<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>> {
        self.bus
            .listen(queue.as_str(), exchange.as_str(), routing_keys.to_vec())
    }

    pub fn close<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>> {
        self.bus.close()
    }
}
