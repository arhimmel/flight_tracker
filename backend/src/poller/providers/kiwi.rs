use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::poller::price_fetcher::{PriceFetcher, PriceResult};

/// Kiwi Tequila API provider.
///
/// Sign up at https://tequila.kiwi.com to obtain a free API key.
/// Set the `KIWI_API_KEY` environment variable before starting the server.
pub struct KiwiProvider {
    client: Client,
    api_key: String,
}

impl KiwiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[derive(Debug, Deserialize)]
struct KiwiResponse {
    data: Vec<KiwiFlight>,
}

#[derive(Debug, Deserialize)]
struct KiwiFlight {
    price: f64,
    // Kiwi returns route segments; we match on flight_no inside route
    route: Vec<KiwiSegment>,
}

#[derive(Debug, Deserialize)]
struct KiwiSegment {
    flight_no: i64,
    #[serde(rename = "operating_carrier")]
    carrier: String,
}

#[async_trait]
impl PriceFetcher for KiwiProvider {
    async fn fetch_price(
        &self,
        flight_number: &str,
        origin: &str,
        destination: &str,
        date: &str,
    ) -> Result<PriceResult> {
        // Kiwi expects date as DD/MM/YYYY
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid date format, expected YYYY-MM-DD"));
        }
        let kiwi_date = format!("{}/{}/{}", parts[2], parts[1], parts[0]);

        let resp = self
            .client
            .get("https://tequila.kiwi.com/v2/search")
            .header("apikey", &self.api_key)
            .query(&[
                ("fly_from", origin),
                ("fly_to", destination),
                ("date_from", &kiwi_date),
                ("date_to", &kiwi_date),
                ("flight_type", "oneway"),
                ("curr", "USD"),
                ("limit", "50"),
            ])
            .send()
            .await
            .context("Failed to reach Kiwi API")?;

        if !resp.status().is_success() {
            return Err(anyhow!("Kiwi API returned status {}", resp.status()));
        }

        let body: KiwiResponse = resp.json().await.context("Failed to parse Kiwi response")?;

        // Extract numeric part from flight number, e.g. "AA123" -> 123
        let numeric: i64 = flight_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0);

        // Find the cheapest itinerary that contains our specific flight number
        let price = body
            .data
            .iter()
            .filter(|f| f.route.iter().any(|s| s.flight_no == numeric))
            .map(|f| f.price)
            .fold(f64::INFINITY, f64::min);

        if price == f64::INFINITY {
            // Fall back to overall cheapest if exact flight not found in results
            let fallback = body
                .data
                .iter()
                .map(|f| f.price)
                .fold(f64::INFINITY, f64::min);

            if fallback == f64::INFINITY {
                return Err(anyhow!("No flights found for {origin}-{destination} on {date}"));
            }

            tracing::warn!(
                flight_number,
                "Exact flight not found in Kiwi results, using route cheapest"
            );
            return Ok(PriceResult { price: fallback, provider: "kiwi".to_string() });
        }

        Ok(PriceResult { price, provider: "kiwi".to_string() })
    }
}
