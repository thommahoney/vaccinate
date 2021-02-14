use std::io::Read;
use std::str;

use chrono::{Duration, Local};
use libflate::gzip::Decoder;
use reqwest;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::errors;
use crate::pushover;

// curl 'https://www.walgreens.com/hcschedulersvc/svc/v1/immunizationLocations/availability' \
//   -H 'content-type: application/json; charset=UTF-8' \
//   --data '{"serviceId":"99","position":{"latitude":XX.XXXXXXX,"longitude":-XX.XXXXXX},"appointmentAvailability":{"startDateTime":"2021-02-15"},"radius":25}'

const APPT_URL: &'static str =
    "https://www.walgreens.com/hcschedulersvc/svc/v1/immunizationLocations/availability";
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

// {"stateName":"New York","stateCode":"NY","zipCode":"14564","radius":25,"days":3}
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct AppointmentAvailabilityResponse {
    appointmentsAvailable: bool,
    days: u8,
    radius: u8,
    stateCode: String,
    stateName: String,
    zipCode: String,
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
        let resp = client
            .post(APPT_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("Accept-Encoding", "gzip")
            .body(search_json)
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                let e = format!("Request error: {:?}", e);
                return errors::report("walgreens", e, &self.config).await;
            }
        };

        let status = resp.status();

        let bytes = resp.bytes().await.unwrap();
        let mut decoder = Decoder::new(&bytes as &[u8]).unwrap();
        let mut buf = Vec::new();
        match decoder.read_to_end(&mut buf) {
            Ok(_) => {}
            Err(e) => {
                let e = format!("Error decoding gzip: {:?}", e);
                return errors::report("walgreens", e, &self.config).await;
            }
        }

        let appt_avail: AppointmentAvailabilityResponse =
            match serde_json::from_slice(&buf as &[u8]) {
                Ok(appt) => appt,
                Err(e) => {
                    let e = format!("Invalid response: {:?}", e);
                    return errors::report("walgreens", e, &self.config).await;
                }
            };

        println!(
            "[walgreens] status = {}, response = {:?}",
            status, appt_avail
        );

        match appt_avail.appointmentsAvailable {
            true => {
                println!(
                    "[walgreens] Appointments available in {}!",
                    appt_avail.zipCode
                );
                pushover::send(
                    "Vaccinate!".to_string(),
                    "Walgreens has availability.".to_string(),
                    self.config.clone(),
                )
                .await
            }
            false => {
                println!(
                    "[walgreens] No appointments available in {}.",
                    appt_avail.zipCode
                );
            }
        }
    }
}
