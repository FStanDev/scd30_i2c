// Copyright 2024, F. Stan
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// This file may not be copied, modified, or distributed
// except according to those terms.

//! SCD30 trait implementing basic SCD30 I2C CO2 sensor operations
//!
//! Operations taken from [interface description](https://sensirion.com/media/documents/D7CEEF4A/6165372F/Sensirion_CO2_Sensors_SCD30_Interface_Description.pdf)
//! //! **IMPORTANT**
//! Current version 0.1.2 contains basics operations, some advanced ones like calibration not yet implemented
//! Pending stuff:
//!
//! - [ ] (De-)Activate Automatic Self-Calibration (ASC)
//! - [ ] Set Forced Recalibration
//! - [ ] Set Temperature Offset
//! - [ ] Altitude Compensation
//! - [ ] Soft reset
//!
//! ## Basic Example
//!
//! Obtaining measurements, co2, temperature and humidity
//!
//!
//!```
//!use scd30_i2c::scd30::Scd30;
//!use std::thread;
//!use std::time::Duration;
//!
//!fn main() {
//!    // Open the I2C device
//!    let mut scd = Scd30::new().unwrap();
//!    let mut counter = 0;
//!    scd.trigger_cont_measurements();
//!
//!    scd.set_measurements_interval(2);
//!
//!    loop {
//!        match scd.get_measurements() {
//!            Ok((a, b, c)) => {
//!                println!("Co2: {} ppm Temp: {} C RH: {} %", a, b, c);
//!                thread::sleep(Duration::from_secs(2));
//!                counter += 1;
//!                println!("{}", counter);
//!            }
//!            Err(e) => {
//!                println!(
//!                    "Error obtaining measurements. More details: {}. Waiting 10 seconds for recovering",
//!                    e
//!                );
//!                thread::sleep(Duration::from_secs(10));
//!            }
//!        }
//!    }
//!}
//!```
//!

/// Trait implementing SCD30 device related operations
pub mod scd30;
