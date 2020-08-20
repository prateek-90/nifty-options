// use reqwest::
// use smol::{io, prelude::*, Unblock};
#[macro_use]
use reqwest::Client;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use option_data::*;
use reqwest::header::*;
use std::time::Duration;
const URL: &str = "https://www.nseindia.com/api/option-chain-indices?symbol=NIFTY";
// const URL1: &str = "https://e01fa23b-60f8-4950-b35c-d70a9b59da17.mock.pstmn.io/stockData";
// const URL2: &str = "https://www.google.com";
use anyhow::Result;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
lazy_static! {
    static ref CURRENT_PE_DATA: Mutex<HashMap<String, ResultData>> = {
        let mut map = HashMap::new();
        Mutex::new(map)
    };
    static ref CURRENT_CE_DATA: Mutex<HashMap<String, ResultData>> = {
        let mut map = HashMap::new();
        Mutex::new(map)
    };
}

mod option_data;
use async_io::Timer;
// use std::env;
#[macro_use]
extern crate clap;
use clap::App;
// use timer::Timer;
fn main() {
    // println!("Hello, world!");
    let yaml = load_yaml!("../config/cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let oi_th: f32 = matches
        .value_of("oi_threshold")
        .unwrap()
        .parse::<f32>()
        .unwrap();
    let pc_th: f32 = matches
        .value_of("percent_threshold")
        .unwrap()
        .parse::<f32>()
        .unwrap();
    // let args: Vec<String> = env::args().collect();
    // let threshold: f32 = &args[1];
    let client = Client::builder()
        .timeout(Duration::from_millis(30000))
        .connect_timeout(Duration::from_millis(30000))
        .default_headers(get_headers())
        .cookie_store(true)
        .build()
        .unwrap();
    execute_job(&client, oi_th, pc_th);
}

async fn sleep(dur: Duration) {
    Timer::new(dur).await;
}

fn execute_job(client: &Client, oi_th: f32, pc_th: f32) {
    // println!("Received call {:?}", Local::now());
    smol::run(async {
        loop {
            match read_data(client).await {
                Ok(stock_data) => {
                    let filtered_data = get_filtered_data(stock_data);
                    // println!("Filtered data {:?}", filtered_data);
                    // println!("Processing data at {:?}", Local::now());
                    process_stock_data(filtered_data, oi_th, pc_th);
                    // use colored::*;
                    // let pe_map = CURRENT_PE_DATA.lock().unwrap();
                    // let ce_map = CURRENT_CE_DATA.lock().unwrap();
                    // for (key, val) in pe_map.clone().into_iter() {
                    //     println!("{}  :: key: {} val: {:?}", "PUT".red(), key, val);
                    // }
                    // for (key, val) in ce_map.clone().into_iter() {
                    //     println!("{} :: key: {} val: {:?}", "CALL".blue(), key, val);
                    // }
                }
                Err(e) => {
                    // eprintln!("{:?} Error reading data {:?}", Local::now(), e)
                }
            };
            sleep(Duration::from_secs(60)).await;
        }
    });
}

fn process_stock_data(stock_data: Vec<OptionBidData>, oi_th: f32, pc_th: f32) {
    let mut pe_map = CURRENT_PE_DATA.lock().unwrap();
    let mut ce_map = CURRENT_CE_DATA.lock().unwrap();
    for stock in stock_data.into_iter() {
        // println!("Stock data {:?}", stock);
        if let Some(bid_data) = stock.CE {
            let result_data = ce_map
                .get(&bid_data.strikePrice.to_string().clone())
                .unwrap_or(&ResultData { openInterest: 0.0 });
            // println!("Prev data {:?}", result_data);
            if result_data.openInterest > bid_data.openInterest {
                let prcnt = calculate_prcnt_diff(bid_data.openInterest, result_data.openInterest);
                // println!(
                //     "{:?} CALL :: Old {:?}, new {:?}, chng % - {:?}",
                //     Local::now(),
                //     result_data.openInterest,
                //     bid_data.openInterest,
                //     prcnt
                // );
                if prcnt >= pc_th {
                    // send alert
                    use colored::*;
                    println!(
                        "{:?} CALL :: Strike price - {} ,OI - {}",
                        Local::now(),
                        &bid_data.strikePrice.to_string().blue().bold(),
                        bid_data.openInterest.to_string().red().bold()
                    );
                }
            } else if result_data.openInterest < bid_data.openInterest
                && bid_data.openInterest >= oi_th
            {
                println!(
                    "{:?} CALL :: Adding {:?} -> {:?} ",
                    Local::now(),
                    bid_data.strikePrice,
                    bid_data.openInterest
                );
                ce_map.insert(
                    bid_data.strikePrice.to_string().clone(),
                    ResultData {
                        openInterest: bid_data.openInterest,
                    },
                );
            }
        }
        if let Some(bid_data) = stock.PE {
            let result_data = pe_map
                .get(&bid_data.strikePrice.to_string().clone())
                .unwrap_or(&ResultData { openInterest: 0.0 });
            if result_data.openInterest > bid_data.openInterest {
                let prcnt = calculate_prcnt_diff(bid_data.openInterest, result_data.openInterest);
                // println!(
                //     "{:?} PUT :: Old {:?}, new {:?}, chng % - {:?}",
                //     Local::now(),
                //     result_data.openInterest,
                //     bid_data.openInterest,
                //     prcnt
                // );
                if prcnt >= pc_th {
                    // send alert - print on console
                    use colored::*;
                    println!(
                        "{:?} PUT :: Strike price - {} ,OI - {}",
                        Local::now(),
                        &bid_data.strikePrice.to_string().blue().bold(),
                        bid_data.openInterest.to_string().red().bold()
                    );
                }
            } else if result_data.openInterest < bid_data.openInterest
                && bid_data.openInterest >= oi_th
            {
                println!(
                    "{:?} PUT :: Adding {:?} -> {:?}",
                    Local::now(),
                    bid_data.strikePrice,
                    bid_data.openInterest
                );
                pe_map.insert(
                    bid_data.strikePrice.to_string().clone(),
                    ResultData {
                        openInterest: bid_data.openInterest,
                    },
                );
            }
        }
    }
}

fn calculate_prcnt_diff(curr: f32, prev: f32) -> f32 {
    ((prev - curr) / prev) * 100.0
}

fn get_filtered_data(optional_stock_data: Option<StockData>) -> Vec<OptionBidData> {
    let mut current_date: DateTime<Utc> = Utc::now();
    if !optional_stock_data.is_none() {
        let stock_data = optional_stock_data.unwrap();
        for i in stock_data.records.expiryDates.into_iter() {
            let date = parse_string_to_date(&i).unwrap();
            if current_date <= date {
                current_date = date;
                break;
            }
        }
        // filter current date data
        let filtered_data: Vec<OptionBidData> = stock_data
            .records
            .data
            .into_iter()
            .filter(|data| data.expiryDate == current_date)
            .collect();

        return filtered_data;
    }
    vec![]
}

async fn read_data(client: &Client) -> anyhow::Result<Option<option_data::StockData>> {
    // println!("Get call started !!! {:?}", Local::now());
    let body = client.get(URL).send().await?.text().await?;
    // println!("Get call over !!! {:?}", Local::now());
    // println!("body = {:?}", body);
    let data: Option<option_data::StockData> = match serde_json::from_str(&body) {
        Ok(data) => data,
        Err(e) => {
            // eprintln!("{:?} Error parsing data {:?}", Local::now(), e);
            return Ok(None);
        }
    };
    // println!("data = {:?}", data);
    Ok(Some(data.unwrap()))
}

fn get_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        REFERER,
        "https://www.nseindia.com/get-quotes/derivatives?symbol=NIFTY"
            .parse()
            .unwrap(),
    );
    headers.insert(
        ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.5".parse().unwrap());
    headers.insert(ACCEPT_ENCODING, "identity".parse().unwrap());
    headers.insert(TE, "Trailers".parse().unwrap());
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:79.0) Gecko/20100101 Firefox/79.0"
            .parse()
            .unwrap(),
    );
    // use cookie::{Cookie, CookieBuilder};
    // use time::OffsetDateTime;
    // let cookie = Cookie::build("ak_bmsc", "foo")
    //     .domain(".nseindia.com")
    //     .expires(OffsetDateTime::now_utc())
    //     .max_age(time::Duration::days(1))
    //     .http_only(true)
    //     .secure(true)
    //     .path("/")
    //     .finish();

    // let c  = Cookie::parse_encoded("ak_bmsc=7049AC9B2487841755DF7CBD9754D5416011A82C052F00008C9A335F1476AB31~plZ/Bla92oiqlBBI+cvygHDFrTb7YtZOsaWW+shN0WUXXe8xCMXB08xWKHtepi91uzjGZvQJiLzoP9m9PkLlq9cwf4b+/BkSYxOla5UlQZNg65S6FWSHhp9sJsm699Avs7hSZJJRKt1bQQ0mx5hBjcblxhoKZxYfqvz6tJ4/Y5vcdXGDZSBMHeixwukWV/mkg6tfUpCa0hUr61h1kkj7dC8KebsIaP1ON8MrRIDLEDe18=").unwrap();
    // println!("cookie {:?}", c);
    // HeaderValue::from_bytes(cookie);
    // headers.insert(COOKIE, Cookie::parse()cookie);
    // headers.insert(COOKIE, "ak_bmsc=2456398701324".parse().unwrap());
    // headers.insert(COOKIE, "ak_bmsc=7049AC9B2487841755DF7CBD9754D5416011A82C052F00008C9A335F1476AB31~plZ/Bla92oiqlBBI+cvygHDFrTb7YtZOsaWW+shN0WUXXe8xCMXB08xWKHtepi91uzjGZvQJiLzoP9m9PkLlq9cwf4b+/BkSYxOla5UlQZNg65S6FWSHhp9sJsm699Avs7hSZJJRKt1bQQ0mx5hBjcblxhoKZxYfqvz6tJ4/Y5vcdXGDZSBMHeixwukWV/mkg6tfUpCa0hUr61h1kkj7dC8KebsIaP1ON8MrRIDLEDe18=".parse().unwrap());
    // headers.insert(COOKIE, "ak_bmsc=50A1BD50C50602BE58BDD75F13FAF20B170BD725AB4C00005568325F4C04ED0F~plUR5UsITZnSidub7fclznl7E72s+xf8nOv0BPS1azHdZGnankR3ttuSw2m6IsKaXbFVLGmPA3tITvv1q7d/DmOTH8Wa2SXP7z7zZZ240Xmll2vrV3qDuaKcjfmjZYOFULtAE9kOfhxDMz6f0dtq+hnIQOOD6WAfaq14dzM4k4k22OpBsYEdR4rLSyf+ETho+rlstGvVBinayugqVFvLXT9yO/BFykRcgqozNtjlw1pgT0GyHB75xq1aa9LHKqSP+h; RT=\"z=1&dm=nseindia.com&si=47e1c864-62bb-4d85-9ef2-4feab2daa6cd&ss=kdprc9ii&sl=2&tt=7m1&bcn=%2F%2F684fc539.akstat.io%2F&ld=d9g&nu=73e64fcfbc7d56d25ffa15b047e3cab2&cl=j1h\"; _ga=GA1.2.156116624.1597139034; _gid=GA1.2.879270015.1597139034; _gat_UA-143761337-1=1; bm_sv=AA5CD1C0645490D64A2D530FEAC90F94~lrtTb6dZ8lfV5ZD5yAnWJnkTzmaz0nfDVRfsnT4DKFXKwcWDpa6eaFbV7/n9AEXE7omkNxJH1lagIXGS2tjswLVFFc0cDdqlrIcIpm4mo4CX5w1uRTmp/pBR71Xlzbb01J1HZLGpIvnfQvKA/HI85yZedUa7by5kZzOrgdu5dqc=; nseQuoteSymbols=[{\"symbol\":\"NIFTY\",\"identifier\":null,\"type\":\"equity\"}]".parse().unwrap());
    headers
}

fn parse_string_to_date(s: &str) -> Option<DateTime<Utc>> {
    let naive_date = NaiveDate::parse_from_str(&s, "%d-%b-%Y").unwrap();
    // Add some default time to convert it into a NaiveDateTime
    let naive_datetime: NaiveDateTime = naive_date.and_hms(0, 0, 0);
    // Add a timezone to the object to convert it into a DateTime<UTC>
    let datetime_utc = DateTime::<Utc>::from_utc(naive_datetime, Utc);
    Some(datetime_utc)
}
