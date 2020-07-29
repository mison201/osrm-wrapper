#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket::State;
use rocket::config::{Config};
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::response::status;

use rocket_contrib::json::JsonValue;

use reqwest::blocking::Client;

extern crate serde;
extern crate serde_json;

use std::time::Duration;

mod model;
mod utils;
mod service;


struct LocalConfig(Config);

#[get("/route/<profile>/<boundary>", format = "text/html")]
fn get_route(config: State<LocalConfig>, profile: String, boundary: String) -> status::Custom<JsonValue> {
    let config = &config.0;

    let limit_timeout: u64 = match config.get_int("limit_timeout") {
        Ok(t) => t as u64,
        _ => 100
    };

    let osrm_url_default = match config.get_str("osrm_url_default") {
        Ok(u) => format!("{}/{}?alternatives=true", u.to_string().as_str(), boundary),
        _ => return utils::format_response("missing osrm_url_default config", Status::BadRequest)
    };

    let osrm_url = match config.get_str("osrm_url") {
        Ok(u) => format!("{}/{}?alternatives=true", u, boundary),
        _ => String::from("")
    };

    let vietbando_api_key: String = match config.get_str("vietbando_api_key") {
        Ok(v) => v.to_string(),
        _ => String::from("")
    };

    let vietbando_url: String = match config.get_str("vietbando_url") {
        Ok(v) => v.to_string(),
        _ => String::from("")
    };

    let client = Client::builder()
        .timeout(Duration::from_millis(limit_timeout))
        .build()
        .unwrap();
    match profile.as_str() {
        "smart" => {
            service::call_smart(
                osrm_url.as_str(), 
                osrm_url_default.as_str(), 
                vietbando_url.as_str(), 
                vietbando_api_key.as_str(),
                boundary.as_str(), &client)
        },
        "osrm" => {
            service::call_osrm(osrm_url.as_str(), osrm_url_default.as_str(), &client)
        },
        "vietbando" => {
            service::call_vietbando(vietbando_url.as_str(), vietbando_api_key.as_str(), boundary.as_str(), &client)
        },
        "google" => {
            let google_url = match config.get_str("google_url") {
                Ok(url) => url.to_string(),
                _ => String::from("")
            };
        
            let google_api_key = match config.get_str("google_api_key") {
                Ok(key) => key.to_string(),
                _ => String::from("")
            };

            service::call_google(google_url.as_str(), google_api_key.as_str(), boundary.as_str(), &client)
        },
        _ => {
            utils::format_response("missing or wrong profile param", Status::BadRequest)
        }
    }
}

#[get("/driving/<boundary>?<alternatives>&<sources>", format = "text/html")]
fn get_driving(config: State<LocalConfig>, boundary: String, alternatives: bool, sources: Option<String>) -> status::Custom<JsonValue> {
    let config = &config.0;
    let osrm_url_default = match config.get_str("osrm_url_default") {
        Ok(u) => format!("{}/{}?alternatives={}", u.to_string().as_str(), boundary, alternatives),
        _ => return utils::format_response("missing osrm_url_default config", Status::BadRequest)
    };

    let osrm_url = match config.get_str("osrm_url") {
        Ok(u) => format!("{}/{}?alternatives={}", u, boundary, alternatives),
        _ => String::from("")
    };

    let limit_timeout: u64 = match config.get_int("limit_timeout") {
        Ok(t) => t as u64,
        _ => 100
    };

    let vietbando_api_key: String = match config.get_str("vietbando_api_key") {
        Ok(v) => v.to_string(),
        _ => String::from("")
    };

    let vietbando_url: String = match config.get_str("vietbando_url") {
        Ok(v) => v.to_string(),
        _ => String::from("")
    };

    let google_url = match config.get_str("google_url") {
        Ok(url) => url.to_string(),
        _ => String::from("")
    };

    let google_api_key = match config.get_str("google_api_key") {
        Ok(key) => key.to_string(),
        _ => String::from("")
    };

    let client = Client::builder()
        .timeout(Duration::from_millis(limit_timeout))
        .build()
        .unwrap();
    
    if let Some(s) = sources {
        for source in s.split(",") {
            if source == "osrm" {
                let result = service::call_osrm(osrm_url.as_str(), osrm_url_default.as_str(), &client);
                if result.0.code == 200 {      
                    return result;
                }
                continue;
            }
            
            if source == "smart" {
                let result = service::call_smart(
                    osrm_url.as_str(), 
                    osrm_url_default.as_str(), 
                    vietbando_url.as_str(), 
                    vietbando_api_key.as_str(),
                    boundary.as_str(), &client);
                if result.0.code >= 400 {
                    continue;
                }
                return result;
            }

            if source == "vietbando" {
                let result = service::call_vietbando(vietbando_url.as_str(), vietbando_api_key.as_str(), boundary.as_str(), &client);
                if result.0.code >= 400 {
                    continue;
                }
                return result;
            }

            if source == "google" {
                let result = service::call_google(google_url.as_str(), google_api_key.as_str(), boundary.as_str(), &client);
                if result.0.code >= 400 {
                    continue;
                }
                return result;
            }
        }
    }

    if let Ok(d) = config.get_str("default_source") {
        if d == "vietbando" {
            return service::call_vietbando(vietbando_url.as_str(), vietbando_api_key.as_str(), boundary.as_str(), &client);
        }

        if d == "smart" {
            return service::call_smart(
                osrm_url.as_str(), 
                osrm_url_default.as_str(), 
                vietbando_url.as_str(), 
                vietbando_api_key.as_str(),
                boundary.as_str(), &client);
        }

        if d == "google" {
            return service::call_google(
                google_url.as_str(), 
                google_api_key.as_str(), 
                boundary.as_str(), &client);
        }
    };
    
    return service::call_osrm(osrm_url.as_str(), osrm_url_default.as_str(), &client)
}

#[catch(404)]
pub fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "resource was not found"
    })
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(AdHoc::on_attach("Local Config", |rocket| {
            println!("Attaching local config.");
            let config = rocket.config().clone();
            Ok(rocket.manage(LocalConfig(config)))
        }))
        .mount("/", routes![get_driving, get_route])
        .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}