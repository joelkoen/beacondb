use actix_web::{error::ErrorInternalServerError, post, web, HttpResponse};
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocationRequest {
    wifi_access_points: Vec<AccessPoint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccessPoint {
    mac_address: MacAddress,
}

#[derive(Debug, Serialize)]
struct LocationResponse {
    location: Location,
    accuracy: f32,
}
#[derive(Debug, Serialize)]
struct Location {
    lat: f32,
    lng: f32,
}

#[post("/v1/geolocate")]
pub async fn service(
    data: web::Json<LocationRequest>,
    pool: web::Data<PgPool>,
) -> actix_web::Result<HttpResponse> {
    let data = data.into_inner();
    let pool = pool.into_inner();

    // TODO: come up with a useful estimation algorithm
    let mut count = 0;
    let mut lat = 0;
    let mut lon = 0;
    for x in data.wifi_access_points {
        let y = query!(
            "select latitude, longitude from wifi_grid where bssid = $1",
            x.mac_address
        )
        .fetch_all(&*pool)
        .await
        .map_err(ErrorInternalServerError)?;
        for y in y {
            println!("{} {} {}", x.mac_address, y.latitude, y.longitude);
            count += 1;
            lat += y.latitude;
            lon += y.longitude;
        }
    }

    if count == 0 {
        return Ok(HttpResponse::NotFound().into());
    } else {
        let lat = lat as f32 / count as f32 / 1000.0;
        let lng = lon as f32 / count as f32 / 1000.0;
        println!("https://openstreetmap.org/search?query={lat}%2C{lng}");
        Ok(HttpResponse::Ok().json(LocationResponse {
            location: Location { lat, lng },
            accuracy: 12.3,
        }))
    }
}