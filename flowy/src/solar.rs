///! This file is a copy of [solar.c](https://github.com/jonls/redshift/blob/master/src/solar.c)
///! from redshift.
///!
///! This module makes extensive use of the
///! [Julian Day notation](https://en.wikipedia.org/wiki/Julian_day)
///! to measure elapsed days between events and in calculations.
///!
///! See also https://en.wikipedia.org/wiki/Sunrise_equation#Complete_calculation_on_Earth
///!
use chrono::{DateTime, Local, NaiveDateTime, Timelike, Utc};
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

const SECS_PER_DAY: f64 = 60.0 * 60.0 * 24.0;
const MINS_PER_DAY: f64 = 60.0 * 24.0;
const DAYS_PER_CENTURY: f64 = 36525.0;

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

/// A Julian day, i.e. the number of days since the beginning of the Julian Period
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct JulianDay {
    days: f64,
}

impl JulianDay {
    const JULIAN_YEAR_OFFSET: f64 = 2_451_545.0;
    const UNIX_EPOCH_OFFSET: f64 = Self::JULIAN_YEAR_OFFSET - (0.3 * DAYS_PER_CENTURY);

    /// Calculates the Julian day from a Unix epoch in seconds.
    fn from_epoch(seconds: f64) -> Self {
        Self {
            days: (seconds / SECS_PER_DAY) + Self::UNIX_EPOCH_OFFSET,
        }
    }

    /// Calculates a Julian day from the number of Julian centuries since J2000.0
    fn from_century(century: f64) -> Self {
        Self {
            days: century * DAYS_PER_CENTURY + Self::JULIAN_YEAR_OFFSET,
        }
    }

    /// Calculates the Unix epoch in seconds from this Julian day
    fn epoch(self) -> f64 {
        SECS_PER_DAY * (self.days - Self::UNIX_EPOCH_OFFSET)
    }

    /// Calculates the number of Julian centuries since J2000.0 from this Julian day.
    /// A Julian century is 36525 days.
    fn century(self) -> f64 {
        (self.days - Self::JULIAN_YEAR_OFFSET) / DAYS_PER_CENTURY
    }

    /// Add a number of Julian days
    fn add(self, days: f64) -> Self {
        Self {
            days: self.days + days,
        }
    }

    /// Subtract a number of Julian days
    fn sub(self, days: f64) -> Self {
        Self {
            days: self.days - days,
        }
    }

    /// Round to whole Julian days
    fn round(self) -> Self {
        Self {
            days: self.days.round(),
        }
    }
}

impl std::ops::Sub for JulianDay {
    type Output = f64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.days - rhs.days
    }
}

#[derive(Debug, Default)]
pub struct Timetable {
    angles: HashMap<SolarTime, f64>,
    date: f64,
    lat: f64,
    lon: f64,
    timetable: HashMap<SolarTime, f64>,
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
        ret.insert(SolarTime::Noon, 0f64.to_radians());
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
        let jd = JulianDay::from_epoch(self.date);

        // Calculate Julian century
        let jdn = jd.round();
        let century: f64 = jdn.century();

        // Calculate apparent solar noon
        let sol_noon: f64 = time_of_solar_noon(century, self.lon);
        let j_noon = jdn.sub(0.5).add(sol_noon / MINS_PER_DAY);
        let t_noon: f64 = j_noon.century();

        // Calulate absolute time of other phenomena
        for st in SolarTime::iterator() {
            let angle: f64 = self.angles.get(&st).unwrap_or(&0.0).to_owned();
            let offset: f64 = time_of_solar_elevation(century, t_noon, self.lat, self.lon, angle);
            ret.insert(st, jdn.sub(0.5).add(offset / MINS_PER_DAY).epoch());
        }

        // Insert solar noon
        ret.insert(SolarTime::Noon, j_noon.epoch());

        // Calculate solar midnight
        ret.insert(SolarTime::Midnight, j_noon.add(0.5).epoch());

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

    /// Returns the time of a solar event contained in the internal `Map`
    /// - st: The SolarTime of interest
    pub fn get(&self, st: &SolarTime) -> std::option::Option<&f64> {
        self.timetable.get(st)
    }

    /// Simple utility function to retrieve only sunset and sunrise times
    /// Returns a tuple (sunrise, sunset) as i64
    pub fn get_sunrise_sunset(&self) -> (i64, i64) {
        // Index into the HashMap using SolarTime Enum
        let sunrise: i64 = self.timetable.get(&SolarTime::Sunrise).unwrap().round() as i64;
        let sunset: i64 = self.timetable.get(&SolarTime::Sunset).unwrap().round() as i64;

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
        let past_midnight: f64 =
            self.timetable.get(&SolarTime::Midnight).unwrap().round() - SECS_PER_DAY;
        let diff_seconds: f64 = self.date - past_midnight;

        (diff_seconds / 60.0).round() as i64
    }
}

/// Calculates the geometric mean longitude of the sun (in radians).
/// The mean logitude of the sun is the ecliptic longitude at wich it would be
/// if its orbit was perfectly circular
/// - century: Julian centuries since J2000.0
fn sun_geom_mean_lon(century: f64) -> f64 {
    let ret: f64 = 280.46646 + century * (36000.76983 + century * 0.0003032);
    ret.rem_euclid(360.0).to_radians()
}

/// Calculates the geometric mean anomaly of the sun (in radians).
/// The mean anomaly is the fraction of the sun's period that has elapsed
/// since the sun passed periapsis
/// - century: Julian centuries since J2000.0
fn sun_geom_mean_anomaly(century: f64) -> f64 {
    let ret: f64 = 357.52911 + century * (35999.05029 - century * 0.0001537);
    ret.to_radians()
}

/// Calculates the eccentricity of the Earth's orbit:
/// returns a parameter from 0 to 1 that determines the amount by which
/// its orbit around the sun deviates from a perfect circle
/// - century: Julian centuries since J2000.0
fn earth_orbit_eccentricity(century: f64) -> f64 {
    0.016708634 - century * (0.000042037 + century * 0.0000001267)
}

/// Calculates the result of the equation of the center for the sun's orbit,
/// which consists in the difference between the sun's position in its elliptical orbit and
/// its position in a circular one, or just the difference between true anomaly and mean anomaly
/// - century: Julian centuries since J2000.0
fn sun_equation_of_center(century: f64) -> f64 {
    let ma: f64 = sun_geom_mean_anomaly(century);
    let center: f64 = ma.sin() * (1.914602 - century * (0.004817 + 0.000014 * century))
        + (ma * 2.0).sin() * (0.019993 - 0.000101 * century)
        + (ma * 3.0).sin() * 0.000289;

    center.to_radians()
}

// Calculates the true longitude of the sun in the elliptical orbit
/// - century: Julian centuries since J2000.0
fn sun_true_lon(century: f64) -> f64 {
    sun_geom_mean_lon(century) + sun_equation_of_center(century)
}

/// Calculates the apparent longitude of the sun (right ascension)
/// - century: Julian centuries since J2000.0
fn sun_apparent_lon(century: f64) -> f64 {
    let term: f64 = 125.04 - 1934.136 * century;
    let true_lon: f64 = sun_true_lon(century);
    let ret: f64 = true_lon.to_degrees() - 0.00569 - 0.00478 * term.to_radians().sin();

    ret.to_radians()
}

/// Calculates the mean obliquity/axial tilt of the Earth's orbit
/// - century: Julian centuries since J2000.0
fn mean_ecliptic_obliquity(century: f64) -> f64 {
    let sec: f64 = 21.448 - century * (46.815 + century * (0.00059 - century * 0.001813));
    let ret: f64 = 23.0 + (26.0 + (sec / 60.0)) / 60.0;

    ret.to_radians()
}

/// Calculates the corrected obliquity/axial tilt of the Earth's orbit
/// - century: Julian centuries since J2000.0
fn obliquity_corrected(century: f64) -> f64 {
    let e_0: f64 = mean_ecliptic_obliquity(century);
    let omega: f64 = 125.04 - century * 1934.136;
    let ret: f64 = e_0.to_degrees() + (0.00256 * omega.to_radians().cos());

    ret.to_radians()
}

/// Calculates the declination (in radians) of the sun's orbit
/// - century: Julian centuries since J2000.0
fn solar_declination(century: f64) -> f64 {
    let e: f64 = obliquity_corrected(century);
    let lambda: f64 = sun_apparent_lon(century);
    let ret: f64 = e.sin() * lambda.sin();

    ret.asin()
}

/// Calculates the difference (in minutes) between true solar time and mean solar time
/// - century: Julian centuries since J2000.0
fn equation_of_time(century: f64) -> f64 {
    let epsilon: f64 = obliquity_corrected(century);
    let l_0: f64 = sun_geom_mean_lon(century);
    let e: f64 = earth_orbit_eccentricity(century);
    let m: f64 = sun_geom_mean_anomaly(century);
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
/// - century: Julian centuries since J2000.0
/// - lon: Longitude of location in degrees
fn time_of_solar_noon(century: f64, lon: f64) -> f64 {
    // First pass uses approximate solar noon to calculate equation of time.
    let t_noon: f64 = JulianDay::from_century(century).sub(lon / 360.0).century();
    let eq_time: f64 = equation_of_time(t_noon);
    let sol_noon: f64 = 720.0 - 4.0 * lon - eq_time;

    // Recalculate using new solar noon
    let t_noon_adj: f64 = JulianDay::from_century(century)
        .sub(0.5)
        .add(sol_noon / MINS_PER_DAY)
        .century();
    let eq_time_adj: f64 = equation_of_time(t_noon_adj);
    let sol_noon_adj: f64 = 720.0 - 4.0 * lon - eq_time_adj;

    sol_noon_adj
}

/// Calculates the time of given apparent solar angular elevation of location on earth.
/// Returns the time difference from mean solar midnight in minutes.
/// - century: Julian centuries since J2000.0
/// - t_noon: Apparent solar noon in Julian centuries since J2000.0
/// - lat: Latitude of location in degrees
/// - lon: Longtitude of location in degrees
/// - elev: Solar angular elevation in radians
fn time_of_solar_elevation(century: f64, t_noon: f64, lat: f64, lon: f64, elev: f64) -> f64 {
    // First pass uses approximate sunrise to calculate equation of time
    let eq_time: f64 = equation_of_time(t_noon);
    let sol_decl: f64 = solar_declination(t_noon);
    let ha: f64 = hour_angle_from_elevation(lat, sol_decl, elev);
    let sol_offset: f64 = 720.0 - 4.0 * (lon + ha.to_degrees()) - eq_time;

    // Recalculate using new sunrise
    let t_rise: f64 = JulianDay::from_century(century)
        .add(sol_offset / MINS_PER_DAY)
        .century();
    let eq_time_adj: f64 = equation_of_time(t_rise);
    let sol_decl_adj: f64 = solar_declination(t_rise);
    let ha_adj: f64 = hour_angle_from_elevation(lat, sol_decl_adj, elev);
    let sol_offset_adj: f64 = 720.0 - 4.0 * (lon + ha_adj.to_degrees()) - eq_time_adj;

    sol_offset_adj
}

/// Calculates the solar angular elevation (in radians) at the given location and time.
/// - century: Julian centuries since J2000.0
/// - lat: Latitude of location
/// - lon: Longitude of location
fn solar_elevation_from_time(century: f64, lat: f64, lon: f64) -> f64 {
    // Minutes from midnight
    let jd = JulianDay::from_century(century);
    let offset: f64 = (jd - jd.round() - 0.5) * MINS_PER_DAY;

    let eq_time: f64 = equation_of_time(century);
    let decl: f64 = solar_declination(century);
    let ha: f64 = ((720.0 - offset - eq_time) / 4.0 - lon).to_radians();

    elevation_from_hour_angle(lat, decl, ha)
}

/// Calculates the solar angular elevation (in degrees) at the given location and time.
/// - epoch: Seconds since unix epoch
/// - lat: Latitude of location
/// - lon: Longitude of location
/// - Return: Solar angular elevation in degrees
pub fn solar_elevation(epoch: f64, lat: f64, lon: f64) -> f64 {
    let jd = JulianDay::from_epoch(epoch);
    let ret: f64 = solar_elevation_from_time(jd.century(), lat, lon);

    ret.to_degrees()
}

/// Converts UNIX seconds to a human readable format (HH:MM:ss)
/// - time: absolute datetime (in epoch seconds) to convert
pub fn unix_to_local(time: i64) -> DateTime<Local> {
    let naive: NaiveDateTime = NaiveDateTime::from_timestamp(time, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    let converted: DateTime<Local> = DateTime::from(datetime);
    // let newdate: String = converted.format("%H:%M:%S").to_string();

    // Return the time in string type
    converted
}

pub fn time_to_minutes(time: String) -> u32 {
    let time = chrono::NaiveTime::parse_from_str(&time, "%H:%M:%S").unwrap();
    let h1 = time.hour();
    let m1 = time.minute();
    h1 * 60 + m1
}
