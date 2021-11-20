use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;

    let mut d0 = gpio.get(17)?.into_output();
    let mut d3 = gpio.get(27)?.into_output();
    let mut d1 = gpio.get(22)?.into_output();
    let mut d2 = gpio.get(23)?.into_output();

    let mut modsel = gpio.get(24)?.into_output();
    let mut enable = gpio.get(25)?.into_output();

    // disable the transmitter
    enable.set_low();

    // set modselector to low for ASK mode
    modsel.set_low();

    // turn on socket 1
    d0.set_high();
    d1.set_high();
    d2.set_high();
    d3.set_high();

    // let it settle, encoder requires this
    thread::sleep(Duration::from_millis(100));

    // enable the modulator
    enable.set_high();

    // enable it for a bit
    thread::sleep(Duration::from_millis(250));

    // disable it again
    enable.set_low();

    Ok(())
}
