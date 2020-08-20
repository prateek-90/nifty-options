use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct OptionBidData {
    pub strikePrice: f32,
    #[serde(with = "my_date_format")]
    pub expiryDate: DateTime<Utc>,
    // #[serde(rename = "call")]
    pub CE: Option<BidData>,
    // #[serde(rename = "put")]
    pub PE: Option<BidData>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct BidData {
    pub strikePrice: f32,
    #[serde(with = "my_date_format")]
    pub expiryDate: DateTime<Utc>,
    pub openInterest: f32,
    pub underlyingValue: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Records {
    pub expiryDates: Vec<String>,
    pub data: Vec<OptionBidData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StockData {
    pub records: Records,
}

mod my_date_format {
    use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%d-%b-%Y";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let naive_date = NaiveDate::parse_from_str(&s, "%d-%b-%Y").unwrap();
        // Add some default time to convert it into a NaiveDateTime
        let naive_datetime: NaiveDateTime = naive_date.and_hms(0, 0, 0);
        // Add a timezone to the object to convert it into a DateTime<UTC>
        let datetime_utc = DateTime::<Utc>::from_utc(naive_datetime, Utc);
        Ok(datetime_utc)
        // Utc.datetime_from_str(&s, FORMAT)
        //     .map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultData {
    pub openInterest: f32,
}
