use std::io::Read;
use std::str;

use chrono::{Duration, Local};
use libflate::gzip::Decoder;
use reqwest;
use serde::Serialize;

use crate::config;

// curl 'https://www.walgreens.com/hcschedulersvc/svc/v1/immunizationLocations/availability' \
//   -H 'content-type: application/json; charset=UTF-8' \
//   --data '{"serviceId":"99","position":{"latitude":XX.XXXXXXX,"longitude":-XX.XXXXXX},"appointmentAvailability":{"startDateTime":"2021-02-15"},"radius":25}'

const APPT_URL: &'static str = "https://www.walgreens.com/hcschedulersvc/svc/v1/immunizationLocations/availability";
const RADIUS: u8 = 25;
const SERVICE_ID: &'static str = "99";
const START_DATE_TIME_FORMAT: &'static str = "%Y-%m-%d"; // 2021-02-14

#[derive(Serialize)]
struct PositionInput {
    latitude: f32,
    longitude: f32,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct AppointmentAvailabilityInput {
    startDateTime: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct SearchInput {
    serviceId: String,
    position: PositionInput,
    appointmentAvailability: AppointmentAvailabilityInput,
    radius: u8,
}

impl SearchInput {
    fn new(address: &config::Address) -> SearchInput {
        let now = Local::now();
        let tomorrow = now + Duration::days(1);
        SearchInput {
            serviceId: String::from(SERVICE_ID),
            position: PositionInput {
                latitude: address.latitude,
                longitude: address.longitude,
            },
            appointmentAvailability: AppointmentAvailabilityInput {
                startDateTime: tomorrow.format(START_DATE_TIME_FORMAT).to_string(),
            },
            radius: RADIUS,
        }
    }
}

pub struct Provider {
    config: config::Config,
}

impl Provider {
    pub fn new(config: config::Config) -> Self {
        Provider { config }
    }

    pub async fn perform(&self) {
        println!("[walgreens] perform");

        let search_input = SearchInput::new(&self.config.address);
        let search_json = serde_json::to_string(&search_input).unwrap();

        println!("[walgreens] request JSON: {}", search_json);

        let client = reqwest::Client::new();
        let resp = client.post(APPT_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("Accept-Encoding", "gzip")
            .body(search_json)
            .send()
            .await;

        match resp {
            Ok(r) => {
                let status = r.status();

                let bytes = r.bytes().await.unwrap();
                let mut decoder = Decoder::new(&bytes as &[u8]).unwrap();
                let mut buf = Vec::new();
                match decoder.read_to_end(&mut buf) {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("[walgreens] Error decoding gzip {:?}", e);
                        return;
                    }
                }
                let text = match String::from_utf8(buf) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("[walgreens] Invalid utf-8 bytes: {:?}", e);
                        return;
                    }
                };

                println!("[walgreens] status = {}, response = {}", status, text);
            },
            Err(e) => {
                eprintln!("[walgreens] Request error: {:?}", e);
            }
        }
    }
}
