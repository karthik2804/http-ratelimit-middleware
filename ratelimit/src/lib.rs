use std::time::{SystemTime, UNIX_EPOCH};
use std::u64;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use spin_sdk::http::{Headers, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;

#[http_component]
async fn middleware(request: IncomingRequest, output: ResponseOutparam) {
    let key = match get_key(&request) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("error getting limiting id: {e}");
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
            output.set(response);
            return;
        }
    };

    match should_rate_limit(key) {
        Ok(val) => match val {
            true => {
                let response = OutgoingResponse::new(429, &Headers::new(&[]));
                output.set(response);
                return;
            }
            false => {
                wasi::http::incoming_handler::handle(request, output.into_inner());
            }
        },
        Err(e) => {
            println!("error getting limiting id: {e}");
            let response = OutgoingResponse::new(500, &Headers::new(&[]));
            output.set(response);
            return;
        }
    }
}

fn get_key(request: &IncomingRequest) -> anyhow::Result<String> {
    let header_key = spin_sdk::variables::get("header_key")?;
    let mut id = request.headers().get(&header_key).into_iter();
    let header = &id.next().ok_or(anyhow::anyhow!("missing host header"))?;
    let mut key = String::from_utf8_lossy(header).to_string();
    if key == "" {
        bail!("Key is empty")
    }
    if header_key == "spin-client-addr" {
        if let Some((ip, _)) = key.split_once(':') {
            key = ip.to_owned();
        }
    }
    Ok(key)
}

#[derive(Serialize, Deserialize, Debug)]
struct RateLimitEntry {
    count: u64,
    expiry_timestamp: u64,
}

fn should_rate_limit(key: String) -> anyhow::Result<bool> {
    let rate_limit_period = spin_sdk::variables::get("rate_limit_period")?.parse::<u64>()?;
    let rate_limit_count: u64 = spin_sdk::variables::get("rate_limit_count")?.parse::<u64>()?;

    let store = spin_sdk::key_value::Store::open_default()?;
    let ratelimit_entry: Option<RateLimitEntry> = store.get_json(key.clone())?;
    let duration_since_epoch: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let mut entry = match ratelimit_entry {
        Some(entry) => entry,
        None => {
            // If no entry this is the first request in this entry
            RateLimitEntry {
                count: rate_limit_count,
                expiry_timestamp: duration_since_epoch + rate_limit_period,
            }
        }
    };

    // Rate limit logic

    // reset limits (refresh when in a new period)
    if entry.expiry_timestamp < duration_since_epoch {
        entry.count = rate_limit_count;
        entry.expiry_timestamp = duration_since_epoch + rate_limit_period;
    }

    // check limits
    if entry.count > 0 {
        entry.count -= 1;
        store.set_json(key, &entry)?;
        return Ok(false);
    }

    Ok(true)
}

wit_bindgen::generate!({
    runtime_path: "::spin_sdk::wit_bindgen::rt",
    world: "wasi-http-import",
    path: "wit",
    with: {
        "wasi:http/types@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::http::types,
        "wasi:io/streams@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::io::streams,
        "wasi:io/poll@0.2.0-rc-2023-10-18": spin_sdk::wit::wasi::io::poll,
    }
});
