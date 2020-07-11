/// This file is a copy of [solar.c](https://github.com/jonls/redshift/blob/master/src/solar.c) from redshift.
///
/// This module makes extensive use of the [Julian Day notation](https://en.wikipedia.org/wiki/Julian_day)
/// to measure elapsed days between events and in calculations.
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use std::collections::HashMap;

/* Ported from javascript code by U.S. Department of Commerce,
National Oceanic & Atmospheric Administration:
http://www.srrb.noaa.gov/highlights/sunrise/calcdetails.html
It is based on equations from "Astronomical Algorithms" by
Jean Meeus. */

/// Model of atmospheric refraction near horizon (in degrees).
const ATM_REFRAC: f64 = 0.833;

const ASTRO_TWILIGHT_ELEV: f64 = -18.0;
const NAUT_TWILIGHT_ELEV: f64 = -12.0;
const CIVIL_TWILIGHT_ELEV: f64 = -6.0;
const DAYTIME_ELEV: f64 = 0.0 - ATM_REFRAC;

/// Representation of various times of day in the solar cycle
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub enum SolarTime {
    Noon = 0,
    Midnight,
    AstroDawn,
    NautDawn,
    CivilDawn,
    Sunrise,
    Sunset,
    CivilDusk,
    NautDusk,
    AstroDusk,
}

impl SolarTime {
    pub fn iterator() -> impl Iterator<Item = SolarTime> {
        [
            SolarTime::Noon,
            SolarTime::Midnight,
            SolarTime::AstroDawn,
            SolarTime::NautDawn,
            SolarTime::CivilDawn,
            SolarTime::Sunrise,
            SolarTime::Sunset,
            SolarTime::CivilDusk,
            SolarTime::NautDusk,
            SolarTime::AstroDusk,
        ]
        .iter()
        .copied()
    }
}

#[derive(Debug)]
pub struct Timetable {
    angles: HashMap<SolarTime, f64>,
    date: f64,
    lat: f64,
    lon: f64,
    timetable: HashMap<SolarTime, f64>,
}

impl Default for Timetable {
    fn default() -> Self {
        Self {
            angles: HashMap::new(),
            date: 0.0,
            lat: 0.0,
            lon: 0.0,
            timetable: HashMap::new(),
        }
    }
}

impl Timetable {
    /// Generates a `Map<SolarTime, f64>` contaning, for each part of the day, the azimuth angle of the sun
    fn generate_time_angles(&self) -> HashMap<SolarTime, f64> {
        let mut ret: HashMap<SolarTime, f64> = HashMap::new();
        ret.insert(
            SolarTime::AstroDawn,
            (-90.0 + ASTRO_TWILIGHT_ELEV).to_radians(),
        );
        ret.insert(
            SolarTime::NautDawn,
            (-90.0 + NAUT_TWILIGHT_ELEV).to_radians(),
        );
        ret.insert(
            SolarTime::CivilDawn,
            (-90.0 + CIVIL_TWILIGHT_ELEV).to_radians(),
        );
        ret.insert(SolarTime::Sunrise, (-90.0 + DAYTIME_ELEV).to_radians());
        ret.insert(SolarTime::Noon, 0.0_f64.to_radians());
        ret.insert(SolarTime::Sunset, (90.0 - DAYTIME_ELEV).to_radians());
        ret.insert(
            SolarTime::CivilDusk,
            (90.0 - CIVIL_TWILIGHT_ELEV).to_radians(),
        );
        ret.insert(
            SolarTime::NautDusk,
            (90.0 - NAUT_TWILIGHT_ELEV).to_radians(),
        );
        ret.insert(
            SolarTime::AstroDusk,
            (90.0 - ASTRO_TWILIGHT_ELEV).to_radians(),
        );

        ret
    }

    /// Generates a `Map<SolarTime, f64>` which contains for all solar events the epoch (seconds)
    /// at which they will occur, given the current date, latitude and longitude
    fn generate_timetable(&self) -> HashMap<SolarTime, f64> {
        let mut ret: HashMap<SolarTime, f64> = HashMap::new();

        // Calculate Julian day
        let jd = jd_from_epoch(self.date);

        // Calculate Julian century
        let jdn: f64 = jd.round();
        let t: f64 = jcent_from_jd(jdn);

        // Calculate apparent solar noon
        let sol_noon: f64 = time_of_solar_noon(t, self.lon);
        let j_noon: f64 = jdn - 0.5 + sol_noon / 1440.0;
        let t_noon: f64 = jcent_from_jd(j_noon);

        // Calulate absolute time of other phenomena
        for st in SolarTime::iterator() {
            let angle: f64 = self.angles.get(&st).unwrap_or(&0.0).to_owned();
            let offset: f64 = time_of_solar_elevation(t, t_noon, self.lat, self.lon, angle);
            ret.insert(st, epoch_from_jd(jdn - 0.5 + offset / 1440.0));
        }

        // Insert solar noon
        ret.insert(SolarTime::Noon, epoch_from_jd(j_noon));

        // Calculate solar midnight
        ret.insert(SolarTime::Midnight, epoch_from_jd(j_noon + 0.5));

        ret
    }

    /// Constructor for Timetable
    /// - date: A date in Unix epoch (seconds)
    /// - lat: Latitude of location
    /// - lon: Longitude of location
    pub fn new(date: f64, lat: f64, lon: f64) -> Self {
        let mut ret = Self::default();
        ret.angles = ret.generate_time_angles();
        ret.date = date;
        ret.lat = lat;
        ret.lon = lon;
        ret.timetable = ret.generate_timetable();

        ret
    }

    /// Simple function to retrieve only sunset and sunrise times
    /// Returns a tuple (sunrise, sunset)
    pub fn get_sunrise_sunset(&self) -> (String, String) {
        // Index into the HashMap using SolarTime Enum
        let sunrise: String =
            unix_to_normal_time(self.timetable.get(&SolarTime::Sunrise).unwrap().round() as i64);
        let sunset: String =
            unix_to_normal_time(self.timetable.get(&SolarTime::Sunset).unwrap().round() as i64);

        // Return tuple of sunsrise and sunset times
        (sunrise, sunset)
    }

    /// Sets a new date for the timetable and regenerates it with the same coordinates
    /// - epoch: a Unix epoch in seconds
    pub fn set_date(&mut self, epoch: f64) {
        self.date = epoch;
        self.timetable = self.generate_timetable();
    }

    /// Returns a rough (but decently precise) number of minutes passed since the last midnight event
    pub fn minutes_since_midnight(&self) -> i64 {
        let day: f64 = 60.0 * 60.0 * 24.0;
        let past_midnight: f64 = self.timetable.get(&SolarTime::Midnight).unwrap().round() - day;
        let diff_seconds: f64 = self.date - past_midnight;

        (diff_seconds / 60.0).round() as i64
    }
}

/// Calculates the Unix epoch from a Julian day
/// - jd: a Julian day
fn epoch_from_jd(jd: f64) -> f64 {
    86400.0 * (jd - 2440587.5)
}

/// Calculates the Julian day from a Unix epoch
/// - t: a Unix epoch in seconds
fn jd_from_epoch(t: f64) -> f64 {
    (t / 86400.0) + 2440587.5
}

/// Calculates the number of Julian centuries since J2000.0 from a Julian day
/// - jd: a Julian day
fn jcent_from_jd(jd: f64) -> f64 {
    (jd - 2451545.0) / 36525.0
}

/// Calculates a Julian day from the number of Julian centuries since J2000.0
/// - t: Julian centuries since J2000.0
fn jd_from_jcent(t: f64) -> f64 {
    36525.0 * t + 2451545.0
}

/// Calculates the geometric mean longitude of the sun (in radians).
/// The mean logitude of the sun is the ecliptic longitude at wich it would be
/// if its orbit was perfectly circular
/// - t: Julian centuries since J2000.0
fn sun_geom_mean_lon(t: f64) -> f64 {
    let ret: f64 = 280.46646 + t * (36000.76983 + t * 0.0003032);
    ret.rem_euclid(360.0).to_radians()
}

/// Calculates the geometric mean anomaly of the sun (in radians).
/// The mean anomaly is the fraction of the sun's period that has elapsed
/// since the sun passed periapsis
/// - t: Julian centuries since J2000.0
fn sun_geom_mean_anomaly(t: f64) -> f64 {
    let ret: f64 = 357.52911 + t * (35999.05029 - t * 0.0001537);
    ret.to_radians()
}

/// Calculates the eccentricity of the Earth's orbit:
/// returns a parameter from 0 to 1 that determines the amount by which
/// its orbit around the sun deviates from a perfect circle
/// - t: Julian centuries since J2000.0
fn earth_orbit_eccentricity(t: f64) -> f64 {
    0.016708634 - t * (0.000042037 + t * 0.0000001267)
}

/// Calculates the result of the equation of the center for the sun's orbit,
/// which consists in the difference between the sun's position in its elliptical orbit and
/// its position in a circular one, or just the difference between true anomaly and mean anomaly
/// - t: Julian centuries since J2000.0
fn sun_equation_of_center(t: f64) -> f64 {
    let ma: f64 = sun_geom_mean_anomaly(t);
    let center: f64 = ma.sin() * (1.914602 - t * (0.004817 + 0.000014 * t))
        + (ma * 2.0).sin() * (0.019993 - 0.000101 * t)
        + (ma * 3.0).sin() * 0.000289;

    center.to_radians()
}

// Calculates the true longitude of the sun in the elliptical orbit
/// - t: Julian centuries since J2000.0
fn sun_true_lon(t: f64) -> f64 {
    sun_geom_mean_lon(t) + sun_equation_of_center(t)
}

/// Calculates the apparent longitude of the sun (right ascension)
/// - t: Julian centuries since J2000.0
fn sun_apparent_lon(t: f64) -> f64 {
    let term: f64 = 125.04 - 1934.136 * t;
    let true_lon: f64 = sun_true_lon(t);
    let ret: f64 = true_lon.to_degrees() - 0.00569 - 0.00478 * term.to_radians().sin();

    ret.to_radians()
}

/// Calculates the mean obliquity/axial tilt of the Earth's orbit
/// - t: Julian centuries since J2000.0
fn mean_ecliptic_obliquity(t: f64) -> f64 {
    let sec: f64 = 21.448 - t * (46.815 + t * (0.00059 - t * 0.001813));
    let ret: f64 = 23.0 + (26.0 + (sec / 60.0)) / 60.0;

    ret.to_radians()
}

/// Calculates the corrected obliquity/axial tilt of the Earth's orbit
/// - t: Julian centuries since J2000.0
fn obliquity_corrected(t: f64) -> f64 {
    let e_0: f64 = mean_ecliptic_obliquity(t);
    let omega: f64 = 125.04 - t * 1934.136;
    let ret: f64 = e_0.to_degrees() + (0.00256 * omega.to_radians().cos());

    ret.to_radians()
}

/// Calculates the declination (in radians) of the sun's orbit
/// - t: Julian centuries since J2000.0
fn solar_declination(t: f64) -> f64 {
    let e: f64 = obliquity_corrected(t);
    let lambda: f64 = sun_apparent_lon(t);
    let ret: f64 = e.sin() * lambda.sin();

    ret.asin()
}

/// Calculates the difference (in minutes) between true solar time and mean solar time
/// - t: Julian centuries since J2000.0
fn equation_of_time(t: f64) -> f64 {
    let epsilon: f64 = obliquity_corrected(t);
    let l_0: f64 = sun_geom_mean_lon(t);
    let e: f64 = earth_orbit_eccentricity(t);
    let m: f64 = sun_geom_mean_anomaly(t);
    let y: f64 = (epsilon / 2.0).tan().powi(2);

    let eq_result: f64 = y * (l_0 * 2.0).sin() - 2.0 * e * m.sin()
        + 4.0 * e * y * m.sin() * (l_0 * 2.0).cos()
        - 0.5 * y.powi(2) * (l_0 * 4.0).sin()
        - 1.25 * e.powi(2) * (m * 2.0).sin();

    4.0 * eq_result.to_degrees()
}

/// Calculates the hour angle (in radians) at the location for the given angular elevation.
/// - lat: Latitude of location in degrees
/// - decl: Declination in radians
/// - elev: Angular elevation angle in radians
fn hour_angle_from_elevation(lat: f64, decl: f64, elev: f64) -> f64 {
    let term: f64 = (elev.abs().cos() - lat.to_radians().sin() * decl.sin())
        / (lat.to_radians().cos() * decl.cos());
    let omega: f64 = term.acos();

    omega.copysign(-elev)
}

/// Calculates the hour angle (in radians) at the location for the given angular elevation.
/// - lat: Latitude of location in degrees
/// - decl: Declination in radians
/// - ha: Hour angle in radians
fn elevation_from_hour_angle(lat: f64, decl: f64, ha: f64) -> f64 {
    let ret: f64 =
        ha.cos() * lat.to_radians().cos() * decl.cos() + lat.to_radians().sin() * decl.sin();

    ret.asin()
}

/// Calculates the time of apparent solar noon of a location on Earth.
/// Returns the time difference from mean solar midnight in minutes.
/// - t: Julian centuries since J2000.0
/// - lon: Longitude of location in degrees
fn time_of_solar_noon(t: f64, lon: f64) -> f64 {
    // First pass uses approximate solar noon to calculate equation of time.
    let t_noon: f64 = jcent_from_jd(jd_from_jcent(t) - lon / 360.0);
    let eq_time: f64 = equation_of_time(t_noon);
    let sol_noon: f64 = 720.0 - 4.0 * lon - eq_time;

    // Recalculate using new solar noon
    let t_noon_adj: f64 = jcent_from_jd(jd_from_jcent(t) - 0.5 + sol_noon / 1440.0);
    let eq_time_adj: f64 = equation_of_time(t_noon_adj);
    let sol_noon_adj: f64 = 720.0 - 4.0 * lon - eq_time_adj;

    sol_noon_adj
}

/// Calculates the time of given apparent solar angular elevation of location on earth.
/// Returns the time difference from mean solar midnight in minutes.
/// - t: Julian centuries since J2000.0
/// - t_noon: Apparent solar noon in Julian centuries since J2000.0
/// - lat: Latitude of location in degrees
/// - lon: Longtitude of location in degrees
/// - elev: Solar angular elevation in radians
fn time_of_solar_elevation(t: f64, t_noon: f64, lat: f64, lon: f64, elev: f64) -> f64 {
    // First pass uses approximate sunrise to calculate equation of time
    let eq_time: f64 = equation_of_time(t_noon);
    let sol_decl: f64 = solar_declination(t_noon);
    let ha: f64 = hour_angle_from_elevation(lat, sol_decl, elev);
    let sol_offset: f64 = 720.0 - 4.0 * (lon + ha.to_degrees()) - eq_time;

    // Recalculate using new sunrise
    let t_rise: f64 = jcent_from_jd(jd_from_jcent(t) + sol_offset / 1440.0);
    let eq_time_adj: f64 = equation_of_time(t_rise);
    let sol_decl_adj: f64 = solar_declination(t_rise);
    let ha_adj: f64 = hour_angle_from_elevation(lat, sol_decl_adj, elev);
    let sol_offset_adj: f64 = 720.0 - 4.0 * (lon + ha_adj.to_degrees()) - eq_time_adj;

    sol_offset_adj
}

/// Calculates the solar angular elevation (in radians) at the given location and time.
/// - t: Julian centuries since J2000.0
/// - lat: Latitude of location
/// - lon: Longitude of location
fn solar_elevation_from_time(t: f64, lat: f64, lon: f64) -> f64 {
    // Minutes from midnight
    let jd: f64 = jd_from_jcent(t);
    let offset: f64 = (jd - jd.round() - 0.5) * 1440.0;

    let eq_time: f64 = equation_of_time(t);
    let decl: f64 = solar_declination(t);
    let ha: f64 = ((720.0 - offset - eq_time) / 4.0 - lon).to_radians();

    elevation_from_hour_angle(lat, decl, ha)
}

/// Calculates the solar angular elevation (in degrees) at the given location and time.
/// - date: Seconds since unix epoch
/// - lat: Latitude of location
/// - lon: Longitude of location
/// - Return: Solar angular elevation in degrees
pub fn solar_elevation(date: f64, lat: f64, lon: f64) -> f64 {
    let jd: f64 = jd_from_epoch(date);
    let ret: f64 = solar_elevation_from_time(jcent_from_jd(jd), lat, lon);

    ret.to_degrees()
}

/// Converts UNIX seconds to a human readable format (HH:MM:ss)
/// - time: absolute datetime (in epoch seconds) to convert
fn unix_to_normal_time(time: i64) -> String {
    let naive: NaiveDateTime = NaiveDateTime::from_timestamp(time, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    let converted: DateTime<Local> = DateTime::from(datetime);
    let newdate: String = converted.format("%H:%M:%S").to_string();

    // Return the time in string type
    newdate
}
