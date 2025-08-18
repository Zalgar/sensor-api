use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use log::{info, error, debug};
use std::io;
use std::thread;
use std::time::Duration;

pub struct BH1750 {
    i2c: I2cdev,
    address: u8,
}

impl BH1750 {
    pub fn new(i2c: I2cdev, address: u8) -> Result<Self, io::Error> {
        let mut sensor = BH1750 { i2c, address };
        
        // Verify sensor is present by reading the device
        sensor.power_on()?;
        
        Ok(sensor)
    }

    pub fn begin(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing BH1750 sensor");
        
        // Power on the sensor
        self.power_on()?;
        
        // Set to continuous high resolution mode
        self.set_mode(0x10)?; // Continuously H-Resolution Mode
        
        // Wait for first measurement
        thread::sleep(Duration::from_millis(180)); // Max measurement time for high res mode
        
        info!("BH1750 sensor initialized successfully");
        Ok(())
    }

    pub fn power_on(&mut self) -> Result<(), io::Error> {
        self.write_command(0x01) // Power On command
    }

    #[allow(dead_code)]
    pub fn power_down(&mut self) -> Result<(), io::Error> {
        self.write_command(0x00) // Power Down command
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) -> Result<(), io::Error> {
        self.write_command(0x07) // Reset command
    }

    pub fn set_mode(&mut self, mode: u8) -> Result<(), io::Error> {
        self.write_command(mode)
    }

    pub fn read_light_level(&mut self) -> Result<f32, io::Error> {
        // Read 2 bytes of data
        let mut buffer = [0u8; 2];
        
        // Retry logic for I2C communication
        for attempt in 1..=3 {
            match self.i2c.read(self.address, &mut buffer) {
                Ok(_) => {
                    let raw_value = ((buffer[0] as u16) << 8) | (buffer[1] as u16);
                    // Convert to lux (standard formula for BH1750 in high resolution mode)
                    let lux = raw_value as f32 / 1.2;
                    info!("Light level: {:.2} lux (raw: {})", lux, raw_value);
                    return Ok(lux);
                },
                Err(e) => {
                    if attempt == 3 {
                        error!("Failed to read light level after {} attempts: {:?}", attempt, e);
                        return Err(io::Error::new(io::ErrorKind::Other, format!("I2C read failed: {:?}", e)));
                    }
                    debug!("Read attempt {} failed, retrying: {:?}", attempt, e);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        unreachable!()
    }

    fn write_command(&mut self, command: u8) -> Result<(), io::Error> {
        // Retry logic for I2C communication
        for attempt in 1..=3 {
            match self.i2c.write(self.address, &[command]) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt == 3 {
                        return Err(io::Error::new(io::ErrorKind::Other, format!("I2C write failed after {} attempts: {:?}", attempt, e)));
                    }
                    debug!("Write attempt {} failed, retrying: {:?}", attempt, e);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        unreachable!()
    }
}
