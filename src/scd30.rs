// Copyright 2024, F. Stan
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// This file may not be copied, modified, or distributed
// except according to those terms.

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::error::Error;
use std::fmt;
use std::io;
use std::{thread, time};

///
///SCD30 error enum, including Io error from
///i2cdev library. ChecksumError when a crc 8
///checksum does not correspond with the calculated
///one. CommunicationError when read or write operations
///fails
///
#[derive(Debug)]
pub enum Scd30Error {
    /// Input/output error
    Io(io::Error),
    /// ChecksumError when the checksum does not correspond to calculated checksum using crc
    /// algorithm
    ChecksumError,
    /// Communication error when the trait tries to read or write to scd30 device
    ComunicationError,
}
///Implementation for Io error to Scd30Error
impl From<io::Error> for Scd30Error {
    fn from(e: io::Error) -> Self {
        Scd30Error::Io(e)
    }
}
///Implementation of display for SCD30Error
impl fmt::Display for Scd30Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Scd30Error::ChecksumError => fmt::Display::fmt("Checksum Error found", f),
            Scd30Error::Io(ref e) => fmt::Display::fmt(e, f),
            Scd30Error::ComunicationError => fmt::Display::fmt("Comunication error with device", f),
        }
    }
}
///Implementation for Error to SCD30
impl Error for Scd30Error {}

/// SCD30 Struct, wraps a LinuxI2CDevice structs
/// and has implemented related SCD30 operations
///
pub struct Scd30 {
    pub i2cdev: LinuxI2CDevice,
}

/// Implementation of SCD30 related
/// operations
///
///
impl Scd30 {
    /// Create a new SCD30 Struct
    ///
    /// Tries to create the device on standard address 0x61.
    /// If fails, return an LinuxI2CError from i2cdev
    ///
    pub fn new() -> Result<Scd30, LinuxI2CError> {
        let device = LinuxI2CDevice::new("/dev/i2c-1", 0x61)?;
        Ok(Scd30 { i2cdev: device })
    }

    /// Checksum checker function
    /// Thanks to [RequestForCoffee](https://github.com/RequestForCoffee)
    /// for the python version of scd30 communication.
    /// This code is an adaptation of the python version.
    /// More info regarding the [algorithm](https://en.wikipedia.org/wiki/Computation_of_cyclic_redundancy_checks)
    ///
    pub fn crc8(message: &Vec<u8>) -> u8 {
        let mut rem = 0xFF;
        let polynomial = 0x31;
        for byte in message {
            rem ^= byte;
            for _ in 0..8 {
                if (rem & 0x80) != 0 {
                    rem = (rem << 1) ^ polynomial;
                } else {
                    rem = rem << 1
                }
                rem &= 0xFF;
            }
        }
        rem
    }

    /// Checks on 4 bytes data if the checksum is correct
    ///
    /// The parameter is a 6 byte array, the first two and the checksum
    /// and the other two with the ckecksum
    ///
    fn check_crc_in_bytes(co2: &[u8]) -> bool {
        //Splited in two two bytes with checksum
        let first_crc = Scd30::crc8(&vec![co2[0], co2[1]]);
        let second_crc = Scd30::crc8(&vec![co2[3], co2[4]]);

        first_crc == co2[2] && second_crc == co2[5]
    }

    /// Checks the firmware version of the SCD30 device.
    /// If fails, return SCD30Error.
    /// Else returns the firmware version.
    ///
    pub fn check_firmware(&mut self) -> Result<u16, Scd30Error> {
        let buffer: [u8; 2] = [0xd1, 0x00];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                // Read data from the selected register
                let mut data_buffer: [u8; 3] = [0; 3];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        if data_buffer[2] == Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) {
                            Ok(u16::from_be_bytes([data_buffer[0], data_buffer[1]]))
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }

            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Trigger the continous measurements for SCD30 device.
    /// If fails return a communication error.
    /// If succeds, does not return anything.
    ///
    pub fn trigger_cont_measurements(&mut self) -> Result<(), Scd30Error> {
        let buffer: [u8; 5] = [0x00, 0x10, 0x00, 0x00, 0x81];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                Ok(())
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Stops the continous measurements for SCD30 device.
    /// If fails return a communication error.
    /// If succeds, does not return anything.
    ///
    pub fn stop_cont_measurements(&mut self) -> Result<(), Scd30Error> {
        let buffer: [u8; 2] = [0x01, 0x01];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                Ok(())
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Sets the measurements interval for the device,
    /// the default is 2 seconds. You can change it using the second parameter
    ///
    pub fn set_measurements_interval(&mut self, seconds: u16) -> Result<(), Scd30Error> {
        let time_in_bytes: [u8; 2] = seconds.to_be_bytes();
        let checksum = Scd30::crc8(&vec![time_in_bytes[0], time_in_bytes[1]]);
        let buffer: [u8; 5] = [0x46, 0x00, time_in_bytes[0], time_in_bytes[1], checksum];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                Ok(())
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Gets if the device is ready for reading
    /// a measurement. If not, returns false.
    /// If error, returns the error.
    pub fn get_data_ready(&mut self) -> Result<bool, Scd30Error> {
        let buffer: [u8; 2] = [0x02, 0x02];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let thirty_millis = time::Duration::from_millis(30);
                thread::sleep(thirty_millis);
                let mut data_buffer: [u8; 3] = [0; 3];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        if Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) == data_buffer[2] {
                            if data_buffer[1] == 0x01 {
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Get CO2, Temperature and Humidity for the device as a f32 tuple.
    /// Checks the checksum for each pair of bytes, if everything ok returns the tuple.
    /// In case of any problem, returns the error.
    pub fn get_measurements(&mut self) -> Result<(f32, f32, f32), Scd30Error> {
        let buffer: [u8; 2] = [0x03, 0x00];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                let mut data_buffer: [u8; 18] = [0; 18];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        let co2_measurement = &data_buffer[0..6];
                        let temp_measurement = &data_buffer[6..12];
                        let rh_measurement = &data_buffer[12..=17];

                        if Scd30::check_crc_in_bytes(co2_measurement)
                            && Scd30::check_crc_in_bytes(temp_measurement)
                            && Scd30::check_crc_in_bytes(rh_measurement)
                        {
                            Ok((
                                f32::from_be_bytes([
                                    co2_measurement[0],
                                    co2_measurement[1],
                                    co2_measurement[3],
                                    co2_measurement[4],
                                ]),
                                f32::from_be_bytes([
                                    temp_measurement[0],
                                    temp_measurement[1],
                                    temp_measurement[3],
                                    temp_measurement[4],
                                ]),
                                f32::from_be_bytes([
                                    rh_measurement[0],
                                    rh_measurement[1],
                                    rh_measurement[3],
                                    rh_measurement[4],
                                ]),
                            ))
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }
    //WIP
    pub fn get_self_calibration_status(&mut self) -> Result<bool, Scd30Error> {
        let buffer: [u8; 2] = [0x53, 0x06];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let thirty_millis = time::Duration::from_millis(30);
                thread::sleep(thirty_millis);
                let mut data_buffer: [u8; 3] = [0; 3];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        if Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) == data_buffer[2] {
                            if data_buffer[1] == 0x01 {
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    //WIP
    pub fn set_self_calibration(&mut self) -> Result<bool, Scd30Error> {
        let buffer: [u8; 2] = [0x53, 0x06];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let thirty_millis = time::Duration::from_millis(30);
                thread::sleep(thirty_millis);
                let mut data_buffer: [u8; 3] = [0; 3];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        if Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) == data_buffer[2] {
                            if data_buffer[1] == 0x01 {
                                Ok(true)
                            } else {
                                Ok(false)
                            }
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Soft reset the sensor device.
    /// If fails, return SCD30Error.
    ///
    pub fn soft_reset(&mut self) -> Result<(), Scd30Error> {
        let buffer: [u8; 2] = [0xd3, 0x04];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                Ok(())
            }

            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Checks the set altitude of the device.
    /// If fails, return SCD30Error.
    /// Else returns the altitue in meters from sea level (0 meters).
    ///
    pub fn check_altitude(&mut self) -> Result<u16, Scd30Error> {
        let buffer: [u8; 2] = [0x51, 0x02];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                // Read data from the selected register
                let mut data_buffer: [u8; 3] = [0; 3];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        if data_buffer[2] == Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) {
                            Ok(u16::from_be_bytes([data_buffer[0], data_buffer[1]]))
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }

            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Sets the altitude for the device.
    /// Altitude is a u16 in meters starting from sea level (0 meters)
    /// If fails returns SCD30Error,
    /// else return nothing.
    /// After the set you can check the saved value to be the same as expected
    pub fn set_altitude(&mut self, altitude: u16) -> Result<(), Scd30Error> {
        let altitude_in_bytes: [u8; 2] = altitude.to_be_bytes();
        let checksum = Scd30::crc8(&vec![altitude_in_bytes[0], altitude_in_bytes[1]]);
        let buffer: [u8; 5] = [
            0x51,
            0x02,
            altitude_in_bytes[0],
            altitude_in_bytes[1],
            checksum,
        ];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                Ok(())
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Checks the temperature offset of the device.
    /// If fails, return SCD30Error.
    /// Else returns the temperature offset in shif ticks, each tick 0.01 Celsius.
    ///
    pub fn check_temperature_offset(&mut self) -> Result<u16, Scd30Error> {
        let buffer: [u8; 2] = [0x54, 0x03];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                // Read data from the selected register
                let mut data_buffer: [u8; 3] = [0; 3];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        if data_buffer[2] == Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) {
                            Ok(u16::from_be_bytes([data_buffer[0], data_buffer[1]]))
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }

            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Sets the temperature offset of the device.
    /// Offset is a u16 correspoding to one tick, each tick is 0.01 Celsius of offset
    /// If fails returns SCD30Error,
    /// else return nothing.
    pub fn set_temperature_offset(&mut self, offset: u16) -> Result<(), Scd30Error> {
        let offset_in_bytes: [u8; 2] = offset.to_be_bytes();
        let checksum = Scd30::crc8(&vec![offset_in_bytes[0], offset_in_bytes[1]]);
        let buffer: [u8; 5] = [0x54, 0x03, offset_in_bytes[0], offset_in_bytes[1], checksum];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                Ok(())
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Checks the forced calibration value of the device.
    /// If fails, return SCD30Error.
    /// Else returns the forced value in ppm units.
    ///
    pub fn get_forced_value(&mut self) -> Result<u16, Scd30Error> {
        let buffer: [u8; 2] = [0x52, 0x04];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                // Read data from the selected register
                let mut data_buffer: [u8; 3] = [0; 3];
                match self.i2cdev.read(&mut data_buffer) {
                    Ok(_) => {
                        if data_buffer[2] == Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) {
                            Ok(u16::from_be_bytes([data_buffer[0], data_buffer[1]]))
                        } else {
                            Err(Scd30Error::ChecksumError)
                        }
                    }
                    Err(_) => Err(Scd30Error::ComunicationError),
                }
            }

            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }

    /// Sets a force recalibration value to the device.
    /// Usually this is use when no time for automatic self calibration is posible.
    /// If fails returns SCD30Error,
    /// else return nothing.
    pub fn set_force_recalibration_value(&mut self, forced_value: u16) -> Result<(), Scd30Error> {
        let forced_value_in_bytes: [u8; 2] = forced_value.to_be_bytes();
        let checksum = Scd30::crc8(&vec![forced_value_in_bytes[0], forced_value_in_bytes[1]]);
        let buffer: [u8; 5] = [
            0x52,
            0x04,
            forced_value_in_bytes[0],
            forced_value_in_bytes[1],
            checksum,
        ];
        match self.i2cdev.write(&buffer) {
            Ok(_) => {
                let ten_millis = time::Duration::from_millis(30);
                thread::sleep(ten_millis);
                Ok(())
            }
            Err(_) => Err(Scd30Error::ComunicationError),
        }
    }
}
