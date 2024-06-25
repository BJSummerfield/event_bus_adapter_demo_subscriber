use futures::stream::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Connection, ConnectionProperties,
};

#[tokio::main]
async fn main() {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://localhost:5672".into());

    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = Connection::connect(&addr, options).await.unwrap();
    let channel = connection.create_channel().await.unwrap();

    let exchange_name = "test_exchange";
    let queue_name = "test_queue";
    let routing_keys = vec!["test.topic"];

    channel
        .exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let queue = channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    for routing_key in &routing_keys {
        channel
            .queue_bind(
                queue.name().as_str(),
                exchange_name,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
    }

    // Set up the consumer
    let consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "test_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    println!("Waiting for messages...");

    // Spawn a task to handle message consumption
    tokio::spawn(async move {
        consumer
            .for_each(|delivery| async {
                match delivery {
                    Ok(delivery) => {
                        let data =
                            std::str::from_utf8(&delivery.data).expect("Invalid UTF-8 sequence");
                        println!("Message received: {}", data);
                        delivery
                            .ack(BasicAckOptions::default())
                            .await
                            .expect("Failed to ack message");
                    }
                    Err(error) => {
                        eprintln!("Error receiving message: {:?}", error);
                    }
                }
            })
            .await;
    });

    std::future::pending::<()>().await;
}
