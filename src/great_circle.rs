// converted from https://github.com/mwgg/GreatCircle/blob/master/GreatCircle.py

/*

def distance(lat1, lon1, lat2, lon2, unit='KM'):
    r = getRadius(unit)
    lat1 = math.radians(lat1)
    lon1 = math.radians(lon1)
    lat2 = math.radians(lat2)
    lon2 = math.radians(lon2)
    londelta = lon2 - lon1
    a = math.pow(math.cos(lat2) * math.sin(londelta), 2) + math.pow(math.cos(lat1) * math.sin(lat2)
        - math.sin(lat1) * math.cos(lat2) * math.cos(londelta), 2)
    b = math.sin(lat1) * math.sin(lat2) + math.cos(lat1) * math.cos(lat2) * math.cos(londelta)
    angle = math.atan2(math.sqrt(a), b)

    return angle * r
*/
pub fn distance(lat1: f64, long1: f64, lat2: f64, long2: f64) -> f64 {
    let long1 = long1.to_radians();
    let long2 = long2.to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let longdelta = long2 - long1;
    let a = (lat2.cos() * longdelta.sin()).powf(2.0)
        + (lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * longdelta.cos()).powf(2.0);
    let b = lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * longdelta.cos();
    let angle = a.sqrt().atan2(b);
    angle * 6371.0
}
/*
def bearing(lat1, lon1, lat2, lon2):
    lat1 = math.radians(lat1)
    lon1 = math.radians(lon1)
    lat2 = math.radians(lat2)
    lon2 = math.radians(lon2)
    londelta = lon2 - lon1
    y = math.sin(londelta) * math.cos(lat2)
    x = math.cos(lat1) * math.sin(lat2) - math.sin(lat1) * math.cos(lat2) * math.cos(londelta)
    brng = math.atan2(y, x)
    brng *= (180 / math.pi)

    if brng < 0:
        brng += 360

    return brng
*/
pub fn bearing(lat1: f64, long1: f64, lat2: f64, long2: f64) -> f64 {
    let long1 = long1.to_radians();
    let long2 = long2.to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let longdelta = long2 - long1;
    let y = longdelta.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * longdelta.cos();
    let mut brng = y.atan2(x);
    brng *= 180.0 / std::f64::consts::PI;
    if brng < 0.0 {
        brng += 360.0;
    }
    brng
}
/*
def destination(lat1, lon1, brng, dt, unit='KM'):
    r = getRadius(unit)
    lat1 = math.radians(lat1)
    lon1 = math.radians(lon1)
    lat3 = math.asin(math.sin(lat1) * math.cos(dt / r) + math.cos(lat1) * math.sin(dt / r) * math.cos(math.radians(brng)))
    lon3 = lon1 + math.atan2(math.sin(math.radians(brng)) * math.sin(dt / r) * math.cos(lat1) , math.cos(dt / r) - math.sin(lat1) * math.sin(lat3))

    return {'LAT': math.degrees(lat3), 'LON': math.degrees(lon3)}
*/

pub fn destination(lat1: f64, long1: f64, brng: f64, dt: f64) -> (f64, f64) {
    let r = 6371.0; // KM
    let lat1 = lat1.to_radians();
    let long1 = long1.to_radians();
    let lat3 = (lat1.sin() * (dt / r).cos()
        + lat1.cos() * (dt / r).sin() * brng.to_radians().cos())
    .asin();
    let lon3 = long1
        + (brng.to_radians().sin() * (dt / r).sin() * lat1.cos())
            .atan2((dt / r).cos() - lat1.sin() * lat3.sin());
    (lat3.to_degrees(), lon3.to_degrees())
}
