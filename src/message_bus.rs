use std::{error::Error, future::Future, pin::Pin};

pub trait MessageBus<'a> {
    fn publish(
        &'a self,
        exchange: &'a str,
        routing_key: &'a str,
        message: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>;

    fn listen(
        &'a self,
        queue: &'a str,
        exchange: &'a str,
        routing_keys: Vec<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>;

    fn close(&'a self) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>;
}

pub enum MessageBrokerExchanges {
    TestExchange,
}

impl MessageBrokerExchanges {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TestExchange => "test_exchange",
        }
    }

    pub fn all() -> Vec<&'static str> {
        vec![Self::TestExchange.as_str()]
    }
}

pub enum MessageBrokerRoutingKeys {
    TestTopic,
    TestTopicTwo,
}

impl MessageBrokerRoutingKeys {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TestTopic => "test.topic",
            Self::TestTopicTwo => "test.topic_two",
        }
    }

    pub fn from_str(routing_key: &str) -> Self {
        match routing_key {
            "test.topic" => Self::TestTopic,
            "test.topic_two" => Self::TestTopicTwo,
            _ => panic!("Invalid routing key"),
        }
    }

    pub fn all() -> Vec<&'static str> {
        vec![Self::TestTopic.as_str(), Self::TestTopicTwo.as_str()]
    }
}

pub enum MessageBrokerQueues {
    TestQueue,
}

impl MessageBrokerQueues {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TestQueue => "test_queue",
        }
    }
}
