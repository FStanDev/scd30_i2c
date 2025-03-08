# SCD30 trait implementing basic SCD30 I2C CO2 sensor operations

Operations taken from [interface description](https://sensirion.com/media/documents/D7CEEF4A/6165372F/Sensirion_CO2_Sensors_SCD30_Interface_Description.pdf)

**IMPORTANT**
Current version 0.1.2 contains basics operations, some advanced ones like calibration not yet implemented

Pending stuff:

- [ ] (De-)Activate Automatic Self-Calibration (ASC)
- [ ] Set Forced Recalibration
- [x] Set Temperature Offset
- [x] Altitude Compensation
- [x] Soft reset

## Basic Example

In your Cargo.toml `scd30_i2c="0.1.2"`

Obtaining measurements, co2, temperature and humidity

```rust
use scd30_i2c::scd30::Scd30;
use std::thread;
use std::time::Duration;

fn main() {
    // Open the I2C device
    let mut scd = Scd30::new().unwrap();
    let mut counter = 0;
    scd.trigger_cont_measurements();

    scd.set_measurements_interval(2);

    loop {
        match scd.get_measurements() {
            Ok((a, b, c)) => {
                println!("Co2: {} ppm Temp: {} C RH: {} %", a, b, c);
                thread::sleep(Duration::from_secs(2));
                counter += 1;
                println!("{}", counter);
            }
            Err(e) => {
                println!(
                    "Error obtaining measurements. More details: {}. Waiting 10 seconds for recovering",
                    e
                );
                thread::sleep(Duration::from_secs(10));
            }
        }
    }
}
```

## Hardware

I made and tested this library using a Raspberry Pi 5 and its I2C capabilities, for other machines running Linux should work, but I don't
have more devices to test

### More Info

This is my first crate made with Rust, so any suggestion is more than welcome.

## Special Thanks

Special thanks to [RequestForCoffe](https://github.com/RequestForCoffee/scd30) for the amazing libray in Python. I used the library
a lot to understand and replicate the code in Rust.
