use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Location {
    Latitude: f64,
    Longitude: f64,
}

impl Location {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self {
            Latitude: lat,
            Longitude: lng,
        }
    }
    
    pub fn set_lat(&mut self, lat: f64) {
        self.Latitude = lat;
    }

    pub fn set_lng(&mut self, lng: f64) {
        self.Longitude = lng;
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct BodyVietBanDo{
    Alternative: i8,
    Distance: i8,
    Duration: i8,
    Geometry: i8,
    Instructions: i8,
    Points: Vec<Location>,
    RouteCriteria: i8,
    Uturn: i8,
    VehicleType: i8,
}

impl BodyVietBanDo {
    pub fn new(points: Vec<Location>) -> Self {
        Self {
            Alternative: 2,
            Distance: 1,
            Duration: 1,
            Geometry: 1,
            Instructions: 1,
            Points: points,
            RouteCriteria: 0,
            Uturn: 1,
            VehicleType: 3,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct VBDError {
    pub ExceptionType: String,
    pub Message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Steps {
    Distances: Vec<i32>,
    Durations: Vec<i32>,
    Indices: Vec<i32>,
    Names: Vec<String>,
    Turns: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Route {
    pub Geometry: String,
    pub Steps: Steps,
    pub Via_Distances: Vec<i32>,
    pub Via_Durations: Vec<i32>,
    pub Via_Indices: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct VBDRoute {
    pub Error: Option<VBDError>,
    pub IsSuccess: bool,
    pub ResponseTime: String,
    pub Value: Option<Value>,
}