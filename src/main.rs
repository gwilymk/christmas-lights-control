use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::{Gpio, OutputPin};

use chrono::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut lights = Energenie::new()?;

    loop {
        let now = chrono::Local::now();
        let (_, sunset) =
            sunrise::sunrise_sunset(52.205338, 0.121817, now.year(), now.month(), now.day());

        let sunset = NaiveDateTime::from_timestamp(sunset, 0).time();

        println!("sunset = {}", sunset);

        if let Some(on) = fixed_state(now) {
            println!(
                "Setting to {} because of fixed state rules",
                if on { "on" } else { "false" }
            );
            lights.set_state(1, on);
        } else if now.time() > sunset {
            println!("Setting to on because after sunset");
            lights.on(1);
        } else {
            println!("Turning off because not after sunset and not forced on");
            lights.off(1);
        }

        thread::sleep(Duration::from_secs(5));
    }
}

fn fixed_state(now: DateTime<Local>) -> Option<bool> {
    let time = now.time();

    let pm11 = NaiveTime::from_hms(23, 0, 0);
    let am7 = NaiveTime::from_hms(7, 0, 0);
    let am9 = NaiveTime::from_hms(9, 0, 0);

    let pm3 = NaiveTime::from_hms(15, 0, 0);
    let pm4 = NaiveTime::from_hms(16, 0, 0);

    // on christmas day, on between 7 and 11
    if now.day() == 25 && now.month() == 12 {
        return Some(time > am7 && time < pm11);
    }

    // on a week day, turn on from 15:00 - 16:00 so that kids coming home from school can see the lights
    if (1..=5).contains(&now.weekday().num_days_from_sunday()) {
        return Some(time > pm3 && time < pm4);
    }

    if time > pm11 || time < am7 {
        Some(false)
    } else if time < am9 {
        Some(true)
    } else {
        None
    }
}

struct Energenie {
    enable: OutputPin,

    setting: [OutputPin; 4],
}

impl Energenie {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let gpio = Gpio::new()?;

        let d0 = gpio.get(17)?.into_output();
        let d3 = gpio.get(27)?.into_output();
        let d1 = gpio.get(22)?.into_output();
        let d2 = gpio.get(23)?.into_output();

        let mut enable = gpio.get(25)?.into_output();
        let mut modsel = gpio.get(24)?.into_output();

        // disable the transmitter
        enable.set_low();

        // set modselector to low for ASK mode
        modsel.set_low();

        Ok(Self {
            enable,
            setting: [d0, d1, d2, d3],
        })
    }

    pub fn set_state(&mut self, id: u8, state: bool) {
        if state {
            self.on(id);
        } else {
            self.off(id);
        }
    }

    pub fn on(&mut self, id: u8) {
        assert!((1..=4).contains(&id), "ID must be within range");

        let code = 16 - id;
        self.send_code(code);
    }

    pub fn off(&mut self, id: u8) {
        assert!((1..=4).contains(&id), "ID must be within range");

        let code = 8 - id;
        self.send_code(code);
    }

    fn send_code(&mut self, code: u8) {
        for i in 0..4 {
            let enable_pin = code & (1 << i);
            self.setting[i].write(enable_pin.into());
        }

        // let it settle, encoder requires this
        thread::sleep(Duration::from_millis(100));

        // enable the modulator
        self.enable.set_high();

        // enable it for a bit
        thread::sleep(Duration::from_millis(250));

        // disable it again
        self.enable.set_low();
    }
}
