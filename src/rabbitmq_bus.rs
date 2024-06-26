use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ConfirmSelectOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
};

use futures::StreamExt;
use std::{error::Error, future::Future, pin::Pin};

use crate::message_bus::{
    MessageBrokerExchanges, MessageBrokerQueues, MessageBrokerRoutingKeys, MessageBus,
};

pub struct RabbitMQBus {
    channel: Channel,
}

impl RabbitMQBus {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://localhost:5672".into());
        let options = ConnectionProperties::default()
            .with_executor(tokio_executor_trait::Tokio::current())
            .with_reactor(tokio_reactor_trait::Tokio);

        let connection = Connection::connect(&addr, options).await?;
        let channel = connection.create_channel().await?;
        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;

        for exchange in MessageBrokerExchanges::all() {
            channel
                .exchange_declare(
                    exchange,
                    lapin::ExchangeKind::Topic,
                    ExchangeDeclareOptions {
                        durable: true,
                        ..ExchangeDeclareOptions::default()
                    },
                    FieldTable::default(),
                )
                .await?;
        }

        Ok(Self { channel })
    }

    async fn setup_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_keys: &[&str],
    ) -> Result<(), Box<dyn Error>> {
        let _ = self
            .channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await?;

        for routing_key in routing_keys {
            self.channel
                .queue_bind(
                    queue,
                    exchange,
                    routing_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await?;
        }

        Ok(())
    }

    async fn start_consumer(&self, queue: &str) -> Result<Consumer, Box<dyn Error>> {
        let consumer = self
            .channel
            .basic_consume(
                queue,
                MessageBrokerQueues::TestQueue.as_str(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(consumer)
    }

    async fn process_delivery(delivery: Result<lapin::message::Delivery, lapin::Error>) {
        match delivery {
            Ok(delivery) => {
                let data = std::str::from_utf8(&delivery.data).expect("Invalid UTF-8 sequence");

                match MessageBrokerRoutingKeys::from_str(delivery.routing_key.as_str()) {
                    MessageBrokerRoutingKeys::TestTopic => {
                        println!("Received TestTopic message: {:?}", data)
                    }
                    MessageBrokerRoutingKeys::TestTopicTwo => {
                        println!("Received TestTopicTwo message: {:?}", data)
                    }
                }

                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("Failed to ack message");
            }
            Err(error) => {
                eprintln!("Error receiving message: {:?}", error);
            }
        }
    }
}

impl<'a> MessageBus<'a> for RabbitMQBus {
    fn publish(
        &'a self,
        exchange: &'a str,
        routing_key: &'a str,
        message: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>> {
        let channel = &self.channel;
        Box::pin(async move {
            channel
                .basic_publish(
                    exchange,
                    routing_key,
                    BasicPublishOptions::default(),
                    message,
                    BasicProperties::default(),
                )
                .await?
                .await?;

            Ok(())
        })
    }

    fn listen(
        &'a self,
        queue: &'a str,
        exchange: &'a str,
        routing_keys: Vec<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>> {
        Box::pin(async move {
            self.setup_queue(queue, exchange, &routing_keys).await?;
            let consumer = self.start_consumer(queue).await?;

            println!("Waiting for messages...");
            tokio::spawn(async move {
                consumer
                    .for_each(|delivery| async {
                        RabbitMQBus::process_delivery(delivery).await;
                    })
                    .await;
            });

            Ok(())
        })
    }

    fn close(&'a self) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>> {
        let channel = &self.channel;
        Box::pin(async move {
            channel.close(200, "Bye").await?;
            Ok(())
        })
    }
}
