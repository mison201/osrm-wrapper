use std::collections::HashMap;
use rocket::config::Value;
use reqwest::blocking::Client;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::JsonValue;
use crate::{utils, model};

pub fn call_smart(osrm_url: &str, osrm_url_default: &str, vietbando_url: &str, vietbando_api_key: &str, boundary: &str, client: &Client) ->status::Custom<JsonValue> {
    let result = call_osrm(osrm_url, osrm_url_default, &client);
    if result.0.code >= 400 {
        return result;
    }
    
    if let Some(routes) = result.1["routes"].as_array() {
        let straight_distance = utils::calc_vincenty_distance(boundary);
        for route in routes {
            if let Some(distance) = route["distance"].as_f64() {
                if distance < straight_distance { // meet condition => call vietbando
                    return call_vietbando(vietbando_url, vietbando_api_key, boundary, &client);
                }
            }
        }
    }    
    return result;
}

pub fn call_google(url: &str, api_key: &str, boundary: &str, client: &Client) -> status::Custom<JsonValue> {
    let completed_url = utils::build_google_url(url, api_key, boundary);
    let resp = client.get(completed_url.as_str())
        .send();

    match resp {
        Ok(res) => {
            if res.status().as_u16() >= 400 {
                return utils::format_response("request fail", Status::BadRequest)
            }

            match res.json::<HashMap<String, Value>>() {
                Ok(res) => {
                    if let Some(routes) = res["routes"].as_array() {
                        if let Some(route) = routes.iter().min_by_key(|r| 
                            r["legs"][0].as_integer().ok_or_else(|| std::u64::MAX)    
                        ) {
                            let distance = match route["legs"][0]["distance"]["value"].as_integer() {
                                Some(d) => d,
                                None => 0,
                            };

                            let duration = match route["legs"][0]["duration"]["value"].as_integer() {
                                Some(d) => d,
                                None => 0,
                            };

                            let geometry = match route["overview_polyline"]["points"].as_str() {
                                Some(g) => g,
                                None => "",
                            };

                            return status::Custom(
                                Status::Ok, 
                                utils::format_response_third_party(geometry, distance as u64, duration as u64, "google"));
                        }
                    }
                },
                Err(err) => {
                    println!("parse google api response fail: {}", err);
                    return utils::format_response("parse response fail", Status::BadRequest)
                }
            }
        },
        Err(err) => {
            println!("call google api fail: {}", err);
            return utils::format_response("request fail", Status::BadRequest)
        }
    }
    return utils::format_response("not do anything", Status::Ok)
}

pub fn call_vietbando(url: &str, api_key: &str, boundary: &str, client: &Client) -> status::Custom<JsonValue> {
    let locations = utils::get_location(boundary);
    let body = model::BodyVietBanDo::new(locations);

    let resp = client.post(url)
        .header("RegisterKey", api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send();

    match resp {
        Ok(res) => {
            if res.status().as_u16() >= 400 {
                return utils::format_response("request fail", Status::BadRequest)
            }

            match res.json::<model::VBDRoute>() {
                Ok(route) => {
                    if route.IsSuccess == false {
                        let err = match route.Error {
                            Some(err) => format!("request fail, type: {}, message: {}", err.ExceptionType, err.Message),
                            None => String::from("request fail"),
                        };
                        return utils::format_response(err.as_str(), Status::BadRequest)
                    }

                    if let Some(v) = route.Value {
                        if let Some(routes) = v["Routes"].as_array() {
                            if let Some(route) = routes.iter().min_by_key(|r| 
                                r["Via_Distances"][1].as_i64().ok_or_else(|| std::u64::MAX)
                            ) {
                                let geometry = match route["Geometry"].as_str() {
                                    Some(g) => g,
                                    None => "",
                                };
                                
                                let geometry = utils::decode_geometry(geometry);

                                let distance = match route["Via_Distances"][1].as_u64() {
                                    Some(d) => d,
                                    None => 0,
                                };

                                let duration = match route["Via_Durations"][1].as_u64() {
                                    Some(d) => d,
                                    None => 0,
                                };

                                return status::Custom(
                                    Status::Ok, 
                                    utils::format_response_third_party(geometry.as_str(), distance, duration, "vietbando"));
                            }
                        }
                    }
                },
                Err(_) => return utils::format_response("parse response fail", Status::BadRequest),
            }
        },
        Err(err) => {
            println!("call vietbando fail: {}", err);
            return utils::format_response("request fail", Status::BadRequest)
        },
    }
    return utils::format_response("not do anything", Status::Ok)
}

pub fn call_osrm(url: &str, url_default: &str, client: &Client) -> status::Custom<JsonValue> {
    if url.is_empty() {
        return call_osrm_url_default(url_default, &client);
    }

    let res = client.get(url).send();
    if let Err(e) = res {
        if e.is_timeout() {
            return call_osrm_url_default(url_default, &client);
        }
        return utils::err_response("request fail", Status::BadRequest, e);
    }

    if let Ok(r) = res {
        if r.status().as_u16() >= 300 {
            return call_osrm_url_default(url_default, &client);
        }

        println!("{:?}", r);
        return match r.json::<HashMap<String, Value>>() {
            Ok(r) => status::Custom(Status::Ok, json!(r)),
            Err(e) => utils::err_response("parsed response fail", Status::BadRequest, e)
        }
    }
    utils::format_response("not do anything", Status::BadRequest)
}

fn call_osrm_url_default(url: &str, client: &Client) -> status::Custom<JsonValue> {
    let res = client.get(url).send();
    match res {
        Ok(v) => {
            match v.json::<HashMap<String, Value>>() {
                Ok(r) => status::Custom(Status::Ok, json!(r)),
                Err(e) => utils::err_response("parsed response fail", Status::BadRequest, e)
            }
        }
        Err(e) => utils::err_response("request fail", Status::BadRequest, e)
    }
}