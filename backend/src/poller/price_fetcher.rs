use anyhow::Result;
use async_trait::async_trait;

/// The result of a single price lookup.
#[derive(Debug, Clone)]
pub struct PriceResult {
    /// Best available price found for the given flight/route/date.
    pub price: f64,
    /// Human-readable name of the provider that returned this result.
    pub provider: String,
}

/// Abstraction over any flight price data provider.
///
/// Add a new provider by implementing this trait and wiring it in `main.rs`
/// via the `PRICE_PROVIDER` environment variable.  The poller depends only on
/// this trait — it never calls a concrete provider directly.
#[async_trait]
pub trait PriceFetcher: Send + Sync {
    /// Fetch the current price for a specific flight on a given date.
    ///
    /// # Arguments
    /// * `flight_number` - IATA flight number, e.g. `"AA123"`
    /// * `origin`        - Departure IATA airport code, e.g. `"JFK"`
    /// * `destination`   - Arrival IATA airport code, e.g. `"LAX"`
    /// * `date`          - ISO 8601 date string, e.g. `"2026-04-15"`
    async fn fetch_price(
        &self,
        flight_number: &str,
        origin: &str,
        destination: &str,
        date: &str,
    ) -> Result<PriceResult>;
}
