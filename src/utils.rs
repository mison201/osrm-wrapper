use reqwest::Error;

use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::JsonValue;


use geo::{Point, vincenty_distance::VincentyDistance};
use polyline::{decode_polyline, encode_coordinates};

use crate::model::Location;

pub fn calc_vincenty_distance(boundary: &str) -> f64{
    let mut distance: f64 = 0.0;

    let geo_points = boundary.split(";").collect::<Vec<&str>>();
    if geo_points.len() <= 1 {
        return distance
    }

    for i in 0..=geo_points.len() - 1 {
        let before_point = match geo_points.get(i) {
            Some(geo_point) => get_point(geo_point),
            None => Point::<f64>::new(0.0, 0.0),
        };

        let after_point = match geo_points.get(i + 1) {
            Some(geo_point) => get_point(geo_point),
            None => break,
        };
        
        match after_point.vincenty_distance(&before_point) {
            Ok(r) => distance += r,
            Err(err) => println!("vincenty distance fail: {}", err),
        }
    }
    distance
}

fn get_lat_lng(point: &str) -> (f64, f64) {
    let lat_lng: Vec<&str> = point.rsplitn(2,",").collect();
    let mut lat = 0.0;
    let mut lng = 0.0;

    if let Some(l) = lat_lng.get(0) {
        if let Ok(l) = l.parse::<f64>() {
            lat = l;
        }
    }

    if let Some(l) = lat_lng.get(1) {
        if let Ok(l) = l.parse::<f64>() {
            lng = l;
        }
    }

    (lat, lng)
}

fn get_point(point: &str) -> Point<f64> {
    let (lat, lng) = get_lat_lng(point);
    Point::<f64>::new(lat, lng)
}

pub fn build_google_url(url: &str, api_key: &str, boundary: &str) -> String {
    let mut completed_url = String::from("");
    let mut way_points: Vec<String> = Vec::new();
    let points: Vec<&str> = boundary.split(";").collect();
    for i in 0..=points.len() - 1 {
        let (lat, lng) = get_lat_lng(points[i]);
        if i == 0 {
            completed_url.push_str(format!("origin={},{}", lat, lng).as_str());
            continue;
        }

        if i == points.len() - 1 {
            completed_url.push_str(format!("&destination={},{}", lat, lng).as_str());
            continue;
        }

        way_points.push(format!("{},{}", lat, lng));
    }

    let way_points = way_points.join("|");
    
    format!("{}?{}&waypoints={}&key={}", url, completed_url, way_points, api_key)
}

pub fn get_location(boundary: &str) -> Vec<Location> {
    let mut locations: Vec<Location> = Vec::new();
    for point in boundary.split(";") {
        let (lat, lng) = get_lat_lng(point);
        locations.push(Location::new(lat, lng));
    }

    locations
}

pub fn format_response_third_party(geometry: &str, distance: u64, duration: u64, source: &str) ->JsonValue {
    json!({
        "code": "Ok",
        "routes": [
            {
                "distance": distance,
                "duration:": duration,
                "geometry": geometry,
            }
        ],
        "source": source
    })
}

pub fn decode_geometry(g: &str) -> String {
    match decode_polyline(g, 6) {
        Ok(lines) => {
            match encode_coordinates(lines, 5) {
                Ok(line) => line,
                Err(err) => {
                    println!("encode_coordinates fail: {}", err);
                    String::from("")
                },
            }
        },
        Err(err) => {
            println!("decode_polyline fail: {}", err);
            String::from("")
        },
    }
}

pub fn format_response(message: &str, status: Status) -> status::Custom<JsonValue> {
    status::Custom(
        status,
        json!({
            "status": status.code,
            "message": message
        })
    )
}

pub fn err_response(message: &str, status: Status, e: Error) -> status::Custom<JsonValue> {
    eprint!("{}: {}", message, e);
    format_response(message, status)
}