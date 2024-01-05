use chrono::DateTime;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Local;
use serde::Deserialize;

const STATION_URL:&str = "https://www.rnv-online.de/rest/departure/2494";

/* ******************** JSON Description ****************************** */
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    pub id: String,
    pub name: String,
    #[serde(alias = "graphQL")]
    pub graph_ql: GraphQL,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQL {
    pub response: GraphQLResponse,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLResponse {
    pub name: String,
    pub journeys: JourneysElement,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JourneysElement {
    pub elements: Vec<Journey>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub line_group: LineGroup,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineGroup {
    pub label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Journey {
    pub line: Line,
    pub canceled: bool,
    pub stops: Vec<StopsElement>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopsElement {
    pub destination_label:   String,
    pub planned_departure:   IsoStringDateTime,
    pub realtime_departure:  IsoStringDateTime,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsoStringDateTime {
    pub iso_string: Option<String>,
}

// Return value
pub struct NextDeparture {
    pub request_time: i64,
    pub outbound_station: String,
    pub outbound_diff: i64,
    pub inbound_station: String,
    pub inbound_diff: i64,
    pub failure: bool,
}

pub fn fetch_data(debug_print : Option<bool>) -> NextDeparture {
    let date = Local::now();
    let st_now = SystemTime::now();
    let seconds = st_now.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let timeString = date.format("%Y-%m-%d %H:%M:%S");
    let url = &format!("{}?datetime={}", STATION_URL, timeString);
    let result = reqwest::blocking::get(url);
    
    let mut return_value = NextDeparture {
        failure  : false,
        outbound_station  : String::from(""),
        outbound_diff : 10000,
        inbound_station : String::from(""),
        inbound_diff : 10000,
        request_time : seconds as i64,
    };

    if result.is_err() {
        println!("Could not read station response {:?}", result.err());
        return_value.failure = true;
        return return_value;
    }
    let text = result.unwrap().text();
    if text.is_err() {
        println!("Could not convert response {:?}", text.err());
        return_value.failure = true;
        return return_value;
    }

    let raw_text = &text.unwrap();
    let body: std::result::Result<Station, serde_json::Error> = serde_json::from_str(&raw_text);

    if body.is_err() {
        println!("Could not parse json {:?}", body.err());  
        println!("------------------------- %< ----------------------------");
        println!("{}", &raw_text);
        println!("------------------------- %< ----------------------------");
        return_value.failure = true;
        return return_value;
    }

    if debug_print.is_some() && debug_print.unwrap() == true {
        println!("----------- Seconds {:} {:} requesting ... -----------", seconds, timeString);
    }
    // parse JSON result.. search of both directions
    let json = body.unwrap();
    
    if debug_print.is_some() && debug_print.unwrap() == true {
        println!("Requesting {:}", json.graph_ql.response.name);
        println!("Elements {:}", json.graph_ql.response.journeys.elements.len() );
        println!("------------------------- %< ----------------------------");
        println!("{}", &raw_text);
        println!("------------------------- %< ----------------------------");
    }

    for el in json.graph_ql.response.journeys.elements {

        if debug_print.is_some() && debug_print.unwrap() == true {
            println!("Requesting {:}", json.graph_ql.response.name);
            println!("Line {:}", el.line.line_group.label);
        }
        for stop in el.stops {
            // use only valid data
            if stop.realtime_departure.iso_string.is_some() && 
                stop.destination_label != "" {
                let txt_departure = stop.realtime_departure.iso_string.unwrap();
                let next_departure = DateTime::parse_from_rfc3339(&txt_departure).unwrap();
                
                let diff = next_departure.timestamp() - (seconds  as i64);
                if debug_print.is_some() && debug_print.unwrap() == true {
                    println!("To      {:} {:} (in {:} seconds)", stop.destination_label, txt_departure, diff );
                }
                
                if stop.destination_label.contains("Rheinau") {
                    if diff <  return_value.outbound_diff {
                        return_value.outbound_station = stop.destination_label;
                        return_value.outbound_diff = diff;
                    } else if debug_print.is_some() && debug_print.unwrap() == true {
                        println!("Unkown diff Stop   {:} {:} (in {:} seconds)", stop.destination_label, txt_departure, diff );
                    }
                } else if stop.destination_label.contains("Hochschule") ||
                            stop.destination_label.contains("Hauptbahnhof") ||
                            stop.destination_label.contains("Schönau") {
                    if diff <  return_value.inbound_diff {
                        return_value.inbound_station = stop.destination_label;
                        return_value.inbound_diff = diff;
                    } else if debug_print.is_some() && debug_print.unwrap() == true {
                        println!("Unkown diff Stop   {:} {:} (in {:} seconds)", stop.destination_label, txt_departure, diff );
                    }
                } else if debug_print.is_some() && debug_print.unwrap() == true {
                    println!("Unkown Stop   {:} {:} (in {:} seconds)", stop.destination_label, txt_departure, diff );
                }
            } else {
                println!("Planned {:} {:?}", stop.destination_label, stop.planned_departure.iso_string)
            }
        }
    }
    if debug_print.is_some() && debug_print.unwrap() == true {
        println!("----------- end of straba.rs -----------");
    }
    return_value
}
