use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::error::Error;
use std::fmt;
use std::io;
use std::{thread, time};

#[derive(Debug)]
pub enum Scd30Error {
    /// Input/output error
    Io(io::Error),
    ChecksumError,
    ComunicationError,
}

impl From<io::Error> for Scd30Error {
    fn from(e: io::Error) -> Self {
        Scd30Error::Io(e)
    }
}

impl fmt::Display for Scd30Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Scd30Error::ChecksumError => fmt::Display::fmt("Checksum Error found", f),
            Scd30Error::Io(ref e) => fmt::Display::fmt(e, f),
            Scd30Error::ComunicationError => fmt::Display::fmt("Comunicarion error with device", f),
        }
    }
}

impl Error for Scd30Error {}

pub struct Scd30 {
    i2cdev: LinuxI2CDevice,
}

impl Scd30 {
    pub fn new() -> Result<Scd30, LinuxI2CError> {
        let device = LinuxI2CDevice::new("/dev/i2c-1", 0x61)?;
        Ok(Scd30 { i2cdev: device })
    }
    //** Checksum checker function */
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

    pub fn check_firmware(&mut self) -> u16 {
        let mut buffer: [u8; 2] = [0xd1, 0x00];
        self.i2cdev.write(&mut buffer).unwrap();
        let ten_millis = time::Duration::from_millis(30);
        thread::sleep(ten_millis);
        // Read data from the selected register
        let mut data_buffer: [u8; 3] = [0; 3];
        self.i2cdev.read(&mut data_buffer).unwrap();
        println!("Data buffer {:?}", data_buffer);
        if data_buffer[2] == Scd30::crc8(&vec![data_buffer[0], data_buffer[1]]) {
            println!("Check correcto");
        } else {
            println!("Fallo en check");
        }
        u16::from_be_bytes([data_buffer[0], data_buffer[1]])
    }

    pub fn trigger_cont_measurements(&mut self) {
        let mut buffer: [u8; 5] = [0x00, 0x10, 0x00, 0x00, 0x81];
        self.i2cdev.write(&mut buffer).unwrap();
        let ten_millis = time::Duration::from_millis(30);
        thread::sleep(ten_millis);
    }

    pub fn stop_cont_measurements(&mut self) {
        let mut buffer: [u8; 2] = [0x01, 0x01];
        self.i2cdev.write(&mut buffer).unwrap();
        let ten_millis = time::Duration::from_millis(30);
        thread::sleep(ten_millis);
    }

    pub fn set_measurements_interval(&mut self, seconds: u16) {
        let time_in_bytes: [u8; 2] = seconds.to_be_bytes();
        let checksum = Scd30::crc8(&vec![time_in_bytes[0], time_in_bytes[1]]);
        let mut buffer: [u8; 5] = [0x46, 0x00, time_in_bytes[0], time_in_bytes[1], checksum];
        self.i2cdev.write(&mut buffer).unwrap();
        let ten_millis = time::Duration::from_millis(30);
        thread::sleep(ten_millis);
    }

    pub fn get_measurements(&mut self) -> Result<(f32, f32, f32), Scd30Error> {
        let mut buffer: [u8; 2] = [0x03, 0x00];
        match self.i2cdev.write(&mut buffer) {
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

    fn check_crc_in_bytes(co2: &[u8]) -> bool {
        //Splited in two two bytes with checksum
        let first_crc = Scd30::crc8(&vec![co2[0], co2[1]]);
        let second_crc = Scd30::crc8(&vec![co2[3], co2[4]]);

        first_crc == co2[2] && second_crc == co2[5]
    }
}
