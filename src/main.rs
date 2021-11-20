use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::{Gpio, OutputPin};

fn main() -> Result<(), Box<dyn Error>> {
    let mut lights = Energenie::new()?;

    lights.on(1);
    thread::sleep(Duration::from_secs(5));
    lights.off(1);

    Ok(())
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

    pub fn on(&mut self, id: u8) {
        assert!(id <= 4 && id >= 1, "ID must be within range");

        let code = 16 - id;
        self.send_code(code);
    }

    pub fn off(&mut self, id: u8) {
        assert!(id <= 4 && id >= 1, "ID must be within range");

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
