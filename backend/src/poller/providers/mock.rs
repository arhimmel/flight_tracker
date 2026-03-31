use anyhow::Result;
use async_trait::async_trait;
use rand::Rng;

use crate::poller::price_fetcher::{PriceFetcher, PriceResult};

/// A mock provider that returns a random price between `min` and `max`.
/// Useful for local development and unit tests without hitting any real API.
pub struct MockProvider {
    pub min_price: f64,
    pub max_price: f64,
}

impl MockProvider {
    pub fn new(min_price: f64, max_price: f64) -> Self {
        Self { min_price, max_price }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new(100.0, 500.0)
    }
}

#[async_trait]
impl PriceFetcher for MockProvider {
    async fn fetch_price(
        &self,
        flight_number: &str,
        _origin: &str,
        _destination: &str,
        _date: &str,
    ) -> Result<PriceResult> {
        let price = rand::thread_rng().gen_range(self.min_price..=self.max_price);
        tracing::debug!(flight_number, price, "MockProvider returning price");
        Ok(PriceResult {
            price,
            provider: "mock".to_string(),
        })
    }
}
