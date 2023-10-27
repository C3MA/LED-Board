use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use env_logger;
use log;
use std::net::UdpSocket;
use clap::Parser;

const PANEL_WIDTH: u32 = 32;
const PANEL_COUNT: u32 = 5;
const IMAGE_WIDTH: u32 = PANEL_COUNT * PANEL_WIDTH;
const IMAGE_HEIGHT: u32 = 40;
const IMAGE_WIDTH_BYTE: u32 = IMAGE_WIDTH / 8;
const IMAGE_LENGTH: usize = (IMAGE_WIDTH_BYTE * IMAGE_HEIGHT) as usize;
const PACKAGE_LENGTH: usize = (IMAGE_LENGTH + 1) as usize;

struct Display {
    window: Window,
    display: SimulatorDisplay<BinaryColor>,
}

impl Display {
    fn new() -> Display {
        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::Default)
            .scale(4)
            .build();
        Display {
            display: SimulatorDisplay::new(Size::new(IMAGE_WIDTH, IMAGE_HEIGHT)),
            window: Window::new("LED Board", &output_settings),
        }
    }
    fn clear(&mut self) {
        self.display.clear(BinaryColor::Off).unwrap();
    }
    fn update(&mut self) {
        self.window.update(&self.display);
    }
    fn has_quit(&mut self) -> bool {
        self.window.events().any(|e| e == SimulatorEvent::Quit)
    }
}

struct DisplayServer {
    socket: UdpSocket,
    display: Display,
}

type ImgData = [u8; IMAGE_LENGTH];

impl DisplayServer {
    fn new(addr: &str, port: i32) -> DisplayServer {
        let addr_full = format!("{addr}:{port}");
        log::info!("Listening on {addr_full}");
        let socket = UdpSocket::bind(addr_full).expect("Couldn't bind");
        DisplayServer {
            socket,
            display: Display::new(),
        }
    }
    fn recv_image(&mut self) -> ImgData {
        let mut data = [0; PACKAGE_LENGTH];
        // Read packages until one is PACKAGE_LENGTH
        loop {
            match self.socket.recv_from(&mut data) {
                Err(error) => panic!("{:}", error),
                Ok((PACKAGE_LENGTH, src_addr)) => {
                    log::debug!("Received image from {}", src_addr);
                    let data = &mut data[1..];
                    for i in 0..(IMAGE_LENGTH as usize) {
                        data[i] = data[i].reverse_bits();
                    }
                    return data.try_into().unwrap();
                }
                Ok((bytes_read, src_addr)) => {
                    log::info!("Responding to status checking message from {}", src_addr);
                    let data = &mut data[..bytes_read];
                    match self.socket.send_to(&data, &src_addr) {
                        Ok(_) => {},
                        Err(e) => log::error!("{:}", e),
                    };
                }
            }
        }
    }
    fn draw_info(&mut self) {
        let content = format!("Listening on {:}", self.socket.local_addr().unwrap());
        let text_style = TextStyleBuilder::new()
            .baseline(Baseline::Middle)
            .alignment(Alignment::Center)
            .build();
        let char_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::with_text_style(
            &content,
            self.display.display.bounding_box().center(),
            char_style,
            text_style,
        )
        .draw(&mut self.display.display)
        .unwrap();
    }
    fn run(&mut self) {
        self.display.clear();
        self.draw_info();
        self.display.update();
        'running: loop {
            self.display.clear();
            let data = self.recv_image();
            let image_raw = ImageRaw::<BinaryColor>::new(&data, IMAGE_WIDTH);
            let image = Image::new(&image_raw, Point::zero());
            image.draw(&mut self.display.display).unwrap();
            self.display.update();
            if self.display.has_quit() {
                break 'running;
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to bind to
    #[arg(short, long, default_value_t=4242)]
    port: i32,

    /// Address to bind to
    #[arg(short, long, default_value_t=String::from("0.0.0.0"))]
    addr: String
}

fn main() {
    let args = Args::parse();
    env_logger::init();
    let mut client = DisplayServer::new("0.0.0.0", args.port);
    client.run();
}
