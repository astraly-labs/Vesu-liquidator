use std::fmt;

use anyhow::Result;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::conversions::hexa_price_to_big_decimal;

pub const DEV_API_URL: &str = "https://api.dev.pragma.build/node/v1/data/";
pub const USD_ASSET: &str = "usd";

#[derive(Deserialize, Debug)]
pub struct OracleApiResponse {
    pub price: String,
    pub decimals: u32,
}

#[derive(Debug, Clone)]
pub struct PragmaOracle {
    http_client: reqwest::Client,
    pub api_url: String,
    pub api_key: String,
    pub aggregation_method: AggregationMethod,
    pub interval: Interval,
}

impl PragmaOracle {
    pub fn new(api_key: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            api_url: DEV_API_URL.to_owned(),
            api_key,
            aggregation_method: AggregationMethod::Median,
            interval: Interval::OneMinute,
        }
    }
}

impl PragmaOracle {
    pub fn fetch_price_url(&self, base: String, quote: String) -> String {
        format!(
            "{}{}/{}?interval={}&aggregation={}",
            self.api_url, base, quote, self.interval, self.aggregation_method
        )
    }

    pub async fn get_dollar_price(&self, asset_name: String) -> Result<BigDecimal> {
        let url = self.fetch_price_url(String::from(asset_name.clone()), USD_ASSET.to_owned());
        let response = self
            .http_client
            .get(url)
            .header("x-api-key", &self.api_key)
            .send()
            .await?;
        let oracle_response = response.json::<OracleApiResponse>().await?;
        Ok(hexa_price_to_big_decimal(
            oracle_response.price.as_str(),
            oracle_response.decimals,
        ))
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
/// Supported Aggregation Methods
pub enum AggregationMethod {
    #[serde(rename = "median")]
    #[default]
    Median,
    #[serde(rename = "mean")]
    Mean,
    #[serde(rename = "twap")]
    Twap,
}

impl fmt::Display for AggregationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AggregationMethod::Median => "median",
            AggregationMethod::Mean => "mean",
            AggregationMethod::Twap => "twap",
        };
        write!(f, "{}", name)
    }
}

/// Supported Aggregation Intervals
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum Interval {
    #[serde(rename = "1min")]
    OneMinute,
    #[serde(rename = "15min")]
    FifteenMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "2h")]
    #[default]
    TwoHours,
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Interval::OneMinute => "1min",
            Interval::FifteenMinutes => "15min",
            Interval::OneHour => "1h",
            Interval::TwoHours => "2h",
        };
        write!(f, "{}", name)
    }
}
