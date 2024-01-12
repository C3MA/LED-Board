use std::{time::Duration, fmt::format};
use std::sync::RwLock;
use paho_mqtt;
use str;
use bit::BitIndex;
use chrono_tz::Europe::Berlin;
use chrono::{DateTime, NaiveDateTime, Utc, Timelike};
use std::time::{SystemTime, UNIX_EPOCH};
use openweathermap::{forecast::Weather, Receiver};
use substring::Substring;
use tinybmp::Bmp;
use core::time;
use embedded_graphics::{
    image::Image,
    mono_font::{iso_8859_1::FONT_6X10, iso_8859_1::FONT_5X8, iso_8859_1::FONT_4X6, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use std::net::UdpSocket;
use std::{env, thread};
use std::io;
use std::process::ExitCode;
use openweathermap::forecast::Forecast;
use straba::NextDeparture;
// This declaration will look for a file named `straba.rs` and will
// insert its contents inside a module named `straba` under this scope
mod straba;

const IMAGE_SIZE_BYTE: usize = (IMAGE_WIDTH_BYTE * IMAGE_HEIGHT) as usize; /* one byte contains 8 LEDs, one in each bit */
const IMAGE_WIDTH: u32 = 5 * 32;
const IMAGE_WIDTH_BYTE: u32 = IMAGE_WIDTH / 8; /* one byte contains 8 LEDs, one in each bit */

const IMAGE_HEIGHT: u32 = 40;
const IMAGE_HEIGHT_BYTE: u32 = 40;
const IMAGE_LENGTH: usize = (IMAGE_WIDTH_BYTE * IMAGE_HEIGHT_BYTE) as usize;
const PACKAGE_LENGTH: usize = (IMAGE_LENGTH + 1) as usize;


struct UdpDisplay {
    image: [u8; IMAGE_SIZE_BYTE],
}

impl OriginDimensions for UdpDisplay {
    fn size(&self) -> Size {
        Size::new(IMAGE_WIDTH, IMAGE_HEIGHT)
    }
}

impl DrawTarget for UdpDisplay {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(
            area.points()
                .zip(colors)
                .map(|(pos, color)| Pixel(pos, color)),
        )
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid(&self.bounding_box(), color)
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            if pixel.0.x < 0 {
                continue;
            }
            if pixel.0.y < 0 {
                continue;
            }

            let x = pixel.0.x as u32;
            let y = pixel.0.y as u32;
            if x > (IMAGE_WIDTH - 1) {
                continue;
            }
            if y > (IMAGE_HEIGHT - 1) {
                continue;
            }

            let y = y as u32;
            let v = pixel.1.is_on();
            //println!("pint {x} {y} is on {v}");
            let offset: usize = (x + y * IMAGE_WIDTH) as usize;

            let subbit: usize = (offset % 8).into();
            let byte_offset: usize = (offset / 8).into();

            let current = &mut self.image[byte_offset];
            current.set_bit(subbit, v);
        }
        return Ok(());
    }
}


fn render_weather(display: &mut UdpDisplay ,data: &Option<Result<Forecast, String>>){
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    match data {
        Some(v) => match v {
            Err(error) => {
                Text::new(&error, Point::new(0, 5), text_style)
                    .draw(display)
                    .unwrap();
                println!("{}", &error);
            }
            Ok(result) => {
                let mut temp:&f64 = &-1_f64;
                if !result.list.is_empty() {
                    let mut max:f64 = 0_f64;
                    let mut best = &result.list[0];
                    for forecast in &result.list {
                        let time_s = forecast.dt;
                        let local_time = NaiveDateTime::from_timestamp_millis(time_s*1000).unwrap();
                        let zoned_time : DateTime<Utc> = DateTime::from_utc(local_time, Utc);
                        let europe_time = zoned_time.with_timezone(&Berlin);
                        
                        let hour = europe_time.hour();
                        let minute = europe_time.minute();

                        let cur_time = chrono::offset::Utc::now();
                        if zoned_time > cur_time {
                            println!("Skipping old result {hour}:{minute} @{time_s}");
                        }

                        temp = &forecast.main.temp;
                        println!("Forecast Temp is {temp}°C");

                        match &forecast.rain {
                            Some(x) => {
                                let rain_v = x.three_hours;
                                println!("Rain at {hour}:{minute} @{time_s} with {rain_v} prior best was {max}");
                                if rain_v > max {
                                    best = forecast;
                                    max = rain_v;
                                }
                            },
                            None    => println!("No rain at {hour}:{minute}"),
                        }
                        
                    }
                    
                    let condition = best.weather[0].to_owned();
                    
                    
                    println!("Weather info: {} desc: {} icon {}", condition.main, condition.description, condition.icon);

                    render_weather_icon(&condition, display);
                    

                    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
                        let temp_string = format!("{temp:.0}°C");
                        Text::new(&temp_string, Point::new((IMAGE_WIDTH-60) as i32, 7), text_style)
                        .draw(display)
                        .unwrap();
                }
            }
        },
        None => {
            Text::new("Waiting for data", Point::new(0, 0), text_style)
                .draw(display)
                .unwrap();
            println!("{}", "no result");
        }
    }
}


fn render_weather_icon(condition: &Weather, display: &mut UdpDisplay ){
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let short_icon_code = condition.icon.substring(0,2);
    let icon_image: Result<Bmp<BinaryColor>, tinybmp::ParseError> = match short_icon_code {
        "01" => {
            Bmp::from_slice(include_bytes!("sun.bmp"))
        },
        "02" => {
            Bmp::from_slice(include_bytes!("few_clouds.bmp"))
        },
        "03" => {
            Bmp::from_slice(include_bytes!("scattered_clouds.bmp"))
        },
        "04" => {
            Bmp::from_slice(include_bytes!("broken_clouds.bmp"))
        },
        "09" => {
            Bmp::from_slice(include_bytes!("shower.bmp"))
        },
        "10" => {
            Bmp::from_slice(include_bytes!("rain.bmp"))
        },
        "11" => {
            Bmp::from_slice(include_bytes!("thunderstorm.bmp"))
        },
        "13" => {
            Bmp::from_slice(include_bytes!("snow.bmp"))
        },
        "50" => {
            Bmp::from_slice(include_bytes!("mist.bmp"))
        },
        _ => {
            println!("Missing icon for {short_icon_code}");
            Text::new(&condition.description, Point::new(0, 0), text_style)
                .draw(display)
                .unwrap();
            return;
        }
    };
    Image::new(&icon_image.unwrap(), Point::new((IMAGE_WIDTH-40) as i32, 0)).draw(display).unwrap();
}

fn render_clock(display: &mut UdpDisplay){
    let time = chrono::offset::Utc::now();
    let europe_time = time.with_timezone(&Berlin);
    let hour = europe_time.hour();
    let minute = europe_time.minute();
    let second = europe_time.second();
    let time = format!("{hour:0>2}:{minute:0>2}:{second:0>2}");
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new(&time, Point::new((1) as i32, 7), text_style)
    .draw(display)
    .unwrap();
}

fn render_strab_partial(display: &mut UdpDisplay, station: &String, diff: i64, height: i32) {
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let mut diff_str = format!("{}min", (diff / 60));
    if diff < 60 {
        diff_str = String::from("sofort");
    }
    let station_short: String;
    if str::len(&station) > 13 {
        station_short = station
            .replace("Straße", "Str")
            .replace("straße", "str")
            .replace("Platz", "Pl")
            .replace("platz", "pl")
            .replace("Hauptbahnhof", "Hbf")
            .replace("Bahnhof", "Bf");
    } else {
        station_short = station.to_string();
    }
    let station_clip: String;
    if str::len(&station_short) > 13 {
        station_clip = station_short[0..12].to_string();
    } else {
        station_clip = station_short.to_string();
    }
    Text::new(&station_clip, Point::new(1, height), text_style)
            .draw(display)
            .unwrap();
    Text::new(&diff_str, Point::new(80, height), text_style)
            .draw(display)
            .unwrap();
}

fn render_strab(display: &mut UdpDisplay, straba_res: &NextDeparture) {
    render_strab_partial(display, &straba_res.outbound_station, straba_res.outbound_diff, 17);
    render_strab_partial(display, &straba_res.inbound_station, straba_res.inbound_diff, 27);
}

// The type we'll use to keep our dynamic list of topics inside the
// MQTT client. Since we want to update it after creating the client,
// we need to wrap the data in a lock, like a Mutex or RwLock.
type UserTopics = RwLock<Vec<String>>;

/////////////////////////////////////////////////////////////////////////////

// Callback for a successful connection to the broker.
// We subscribe to the topic(s) we want here.
fn mqtt_on_connect_success(cli: &paho_mqtt::AsyncClient, _msgid: u16) {
    println!("MQTT | Connection succeeded");
    
    // Subscribe to the desired topic(s).
    cli.subscribe("room/ledboard", paho_mqtt::QOS_0);
}

// Callback for a failed attempt to connect to the server.
// We simply sleep and then try again.
//
// Note that normally we don't want to do a blocking operation or sleep
// from  within a callback. But in this case, we know that the client is
// *not* conected, and thus not doing anything important. So we don't worry
// too much about stopping its callback thread.
fn mqtt_on_connect_failure(cli: &paho_mqtt::AsyncClient, _msgid: u16, rc: i32) {
    println!("MQTT | Connection attempt failed with error code {}.\n", rc);
    thread::sleep(Duration::from_millis(2500));
    cli.reconnect_with_callbacks(mqtt_on_connect_success, mqtt_on_connect_failure);
}

fn send_package(ipaddress: String, 
                data: &Option<Result<Forecast, String>>,
                straba_res: &NextDeparture) {
    let mut package: [u8; PACKAGE_LENGTH] = [0; PACKAGE_LENGTH];

    // Brightness
    package[0] = 128;

    let mut display = UdpDisplay {
        image: [0; IMAGE_SIZE_BYTE],
    };

    if data.is_some() {
        render_weather(&mut display, data);                   
    }

    if straba_res.failure == false {
        render_strab(&mut display, straba_res);
    }

    render_clock(&mut display);


    package[1..PACKAGE_LENGTH].copy_from_slice(&display.image);
    // client need to bind to client port (1 before 4242)
    let socket = UdpSocket::bind("0.0.0.0:14242").expect("couldn't bind to address");
    socket
        .send_to(&package, ipaddress + ":4242")
        .expect("couldn't send data");
}

fn help() {
    println!(
        "usage:
LEDboardClient <ip address>"
    );
    println!("one argument necessary!");
    println!("<ip address>");
    println!("second argument is optional:");
    println!("<ip of mqtt server>");
}

fn check_connection(ipaddress: String) -> bool {
    let device_online;
    // generate a faulty package length
    let mut package: [u8; PACKAGE_LENGTH/2] = [0; PACKAGE_LENGTH/2];
    // client need to bind to client port (1 before 4242)
    let socket = UdpSocket::bind("0.0.0.0:14242").expect("couldn't bind to address");
    socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap(); /* 10 seconds timeout */
    socket
        .send_to(&package, ipaddress + ":4242")
        .expect("couldn't send data");

    // self.recv_buff is a [u8; 8092]
    let answer = socket.recv_from(&mut package);
    match answer {
        Ok((_n, _addr)) => {
            //println!("{} bytes response from {:?} {:?}", n, addr, &package[..n]);
            device_online = true;
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            device_online = false;
        }
        Err(_e) => {
            device_online = false;
        }
    }
    return device_online;
}

fn main_function(ipaddress: String, mqtt: Option<String>) -> ExitCode {
    let mut device_online = check_connection(ipaddress.clone());
    if !device_online {
        println!("{:} not online", &ipaddress);
        return ExitCode::FAILURE;
    }
    let mut mqtt_client: Option<paho_mqtt::AsyncClient> = None;
    if mqtt.is_some() {
        let mqtt_ip: String = mqtt.clone().unwrap();
        // Define the set of options for the create.
        // Use an ID for a persistent session.
        let create_opts = paho_mqtt::CreateOptionsBuilder::new()
        .server_uri(mqtt_ip.clone())
        .client_id("ledboard")
        .finalize();
        // Create a client.
        let local_mqtt = paho_mqtt::AsyncClient::new(create_opts).unwrap();
        
        println!("MQTT | Connecting to {:} MQTT server...", mqtt_ip);
        
        // Define the set of options for the connection.
        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        // Set a closure to be called whenever the client connection is established.
        local_mqtt.set_connected_callback(|_cli: &paho_mqtt::AsyncClient| {
            println!("Connected.");
        });

        // Set a closure to be called whenever the client loses the connection.
        // It will attempt to reconnect, and set up function callbacks to keep
        // retrying until the connection is re-established.
        local_mqtt.set_connection_lost_callback(|cli: &paho_mqtt::AsyncClient| {
            println!("Connection lost. Attempting reconnect.");
            thread::sleep(Duration::from_millis(2500));
            cli.reconnect_with_callbacks(mqtt_on_connect_success, mqtt_on_connect_failure);
        });

        // Attach a closure to the client to receive callback
        // on incoming messages.
        local_mqtt.set_message_callback(|cli, msg| {
            if let Some(msg) = msg {
                let topic = msg.topic();
                let payload_str = msg.payload_str();

                println!("MQTT | {} - {}", topic, payload_str);
            }
        });

        // Define the set of options for the connection
        let lwt = paho_mqtt::Message::new("room/ledboard/lwt", "lost connection", 1);

        // The connect options. Defaults to an MQTT v3.x connection.
        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .will_message(lwt)
            .finalize();

        // Make the connection to the broker
        println!("MQTT | Connecting to the MQTT server...");
        local_mqtt.connect_with_callbacks(conn_opts, mqtt_on_connect_success, mqtt_on_connect_failure);

        // move local instance to global scope
        mqtt_client = Some(local_mqtt);
    }

    let receiver = openweathermap::init_forecast("Mannheim",
    "metric",
    "de",
    "978882ab9dd05e7122ff2b0aef2d3e55",
    60,1);

    let mut last_data = Option::None;
    
    // Test Webcrawler for public transportataion
    let mut straba_res = straba::fetch_data(Some(true));
    println!("{:?} {:?}s", straba_res.outbound_station, straba_res.outbound_diff);
    println!("{:?} {:?}s", straba_res.inbound_station , straba_res.inbound_diff);

    // Render start
    send_package(ipaddress.clone(), &last_data, &straba_res);

    loop {
        let st_now = SystemTime::now();
        let seconds = st_now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let delay = time::Duration::from_millis(500);
        thread::sleep(delay);
        // Only request, if the device is present
        if device_online == true {
            let answer = openweathermap::update_forecast(&receiver);
            match answer {
                Some(_) => {
                    last_data = answer;
                }
                None => {
    
                }
            }
        }

        if (straba_res.request_time + 50) < seconds as i64 {
            device_online = check_connection(ipaddress.clone());
            // request once a minute new data
            if device_online == true {
                straba_res = straba::fetch_data(None);
                println!("Update {:?} {:?}s", straba_res.outbound_station, straba_res.outbound_diff);
                println!("Update {:?} {:?}s", straba_res.inbound_station , straba_res.inbound_diff);
            }
        }

        if device_online == true {
            // Render new image
            send_package(ipaddress.clone(), &last_data, &straba_res);
        }

        // Handle MQTT messages
        
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        // no arguments passed
        1 => {
            // show a help message
            help();
            return ExitCode::SUCCESS;
        }
        // one argument passed
        2 => {
            let ip = &args[1];
            return main_function(ip.to_string(), None);
        }
        // two argument passed
        3 => {
            let ip = &args[1];
            let mqtt = &args[2];
            return main_function(ip.to_string(), Some(mqtt.to_string()));
        }
        // all the other cases
        _ => {
            // show a help message
            help();
            return ExitCode::SUCCESS;
        }
    }
}
