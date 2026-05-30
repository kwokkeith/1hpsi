use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PsiResponse {
    #[serde(rename = "MachineID")]
    pub machine_id: String,

    #[serde(rename = "Categories")]
    pub categories: Vec<String>,

    #[serde(rename = "ChartPM25")]
    pub chart_pm25: Chart,

    #[serde(rename = "Chart1HRPM25")]
    pub chart_1hr_pm25: Chart,

    #[serde(rename = "ChartPM10")]
    pub chart_pm10: Chart,

    #[serde(rename = "ChartSO2")]
    pub chart_so2: Chart,

    #[serde(rename = "ChartO3")]
    pub chart_o3: Chart,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Chart {
    #[serde(rename = "DivId")]
    pub div_id: String,

    #[serde(rename = "North")]
    pub north: RegionSeries,

    #[serde(rename = "South")]
    pub south: RegionSeries,

    #[serde(rename = "East")]
    pub east: RegionSeries,

    #[serde(rename = "West")]
    pub west: RegionSeries,

    #[serde(rename = "Central")]
    pub central: RegionSeries,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegionSeries {
    #[serde(rename = "Data")]
    pub data: Vec<DataPoint>,
}

use chrono::NaiveDateTime;
impl RegionSeries {
    pub fn output(&self) -> Vec<(String, String)> {
        let mut result = vec![];
        for dp in &self.data {
            let dt =
                NaiveDateTime::parse_from_str(&dp.proper_date_time(), "%d %b %Y %I:%M%p").unwrap();

            let iso_date_time = dt.format("%Y-%m-%dT%H:%M:%S.%3f").to_string();

            result.push((iso_date_time, dp.rounded_pm25_string()));
        }
        result
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataPoint {
    pub value: f32,

    #[serde(rename = "valueColor")]
    pub value_color: String,

    pub band: String,

    #[serde(rename = "dateTime")]
    pub date_time: String,
}

impl DataPoint {
    pub fn proper_date_time(&self) -> String {
        self.date_time.replace("PM", ":00PM").replace("AM", ":00AM")
    }

    pub fn rounded_pm25_string(&self) -> String {
        match self.psi_value() {
            Some(v) => v.round().to_string(),
            None => "N/A".to_string(),
        }
    }

    const PSI_BREAKPOINTS: [f32; 8] = [0.0, 50.0, 100.0, 200.0, 300.0, 400.0, 500.0, f32::INFINITY];
    const PM25_BREAKPOINTS: [f32; 8] = [0.0, 12.0, 55.0, 150.0, 250.0, 350.0, 500.0, f32::INFINITY];

    fn psi_value(&self) -> Option<f32> {
        let pm25 = self.value;
        if pm25 > 500.0 {
            return Some(pm25);
        };

        let mut pm25_lower_bound = 0.0;
        let mut pm25_upper_bound = f32::INFINITY;
        let mut psi_lower_bound = 0.0;
        let mut psi_upper_bound = f32::INFINITY;

        for idx in 0..7 {
            pm25_lower_bound = Self::PM25_BREAKPOINTS[idx];
            pm25_upper_bound = Self::PM25_BREAKPOINTS[idx + 1];
            psi_lower_bound = Self::PSI_BREAKPOINTS[idx];
            psi_upper_bound = Self::PSI_BREAKPOINTS[idx + 1];
            if (pm25_lower_bound..=pm25_upper_bound).contains(&pm25) {
                break;
            }
        }

        match Self::ilerp(pm25_lower_bound, pm25_upper_bound, pm25) {
            Some(t) => Some(Self::lerp(psi_lower_bound, psi_upper_bound, t)),
            None => None,
        }
    }

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a * (1.0 - t) + b * t
    }

    fn ilerp(a: f32, b: f32, x: f32) -> Option<f32> {
        match a == b {
            true => None,
            false => Some((x - a) / (b - a)),
        }
    }
}
