use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use log::{info, error, debug};
use std::io;
use std::thread;
use std::time::Duration;

// VEML7700 register addresses
const ALS_CONF_0: u8 = 0x00;
const ALS: u8 = 0x04;
const WHITE: u8 = 0x05;

pub struct VEML7700 {
    i2c: I2cdev,
    address: u8,
}

impl VEML7700 {
    pub fn new(i2c: I2cdev, address: u8) -> Result<Self, io::Error> {
        let sensor = VEML7700 { i2c, address };
        Ok(sensor)
    }

    pub fn begin(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing VEML7700 sensor");
        
        // Configure the sensor
        // ALS_CONF_0: ALS gain 1/8, integration time 100ms, persistence 1, interrupt disable, ALS shutdown disable
        // Bits: [15:13] = Reserved, [12:11] = Gain (00 = 1x), [10:6] = Integration time (00 = 25ms), 
        //       [5:4] = Persistence (00 = 1), [1] = ALS interrupt enable (0 = disable), [0] = ALS shutdown (0 = power on)
        let config: u16 = 0x0000; // Gain 1x, Integration time 25ms, no interrupt, power on
        self.write_register(ALS_CONF_0, config)?;
        
        // Wait for first measurement
        thread::sleep(Duration::from_millis(100));
        
        info!("VEML7700 sensor initialized successfully");
        Ok(())
    }

    pub fn set_gain(&mut self, gain: u16) -> Result<(), io::Error> {
        // Read current config
        let mut config = self.read_register(ALS_CONF_0)?;
        
        // Clear gain bits [12:11] and set new gain
        config &= 0xE7FF; // Clear bits 11-12
        config |= (gain & 0x03) << 11; // Set gain (00=1x, 01=2x, 10=1/8x, 11=1/4x)
        
        self.write_register(ALS_CONF_0, config)
    }

    pub fn set_integration_time(&mut self, it: u16) -> Result<(), io::Error> {
        // Read current config
        let mut config = self.read_register(ALS_CONF_0)?;
        
        // Clear integration time bits [9:6] and set new value
        config &= 0xFC3F; // Clear bits 6-9
        config |= (it & 0x0F) << 6; // Set integration time
        
        self.write_register(ALS_CONF_0, config)
    }

    pub fn read_als(&mut self) -> Result<u16, io::Error> {
        self.read_register(ALS)
    }

    pub fn read_white(&mut self) -> Result<u16, io::Error> {
        self.read_register(WHITE)
    }

    pub fn read_lux(&mut self) -> Result<f32, io::Error> {
        let als_raw = self.read_als()?;
        
        // Read current configuration to determine resolution
        let config = self.read_register(ALS_CONF_0)?;
        let gain = (config >> 11) & 0x03; // Extract gain bits [12:11]
        let integration_time = (config >> 6) & 0x0F; // Extract IT bits [9:6]
        
        // Calculate lux based on gain and integration time
        // VEML7700 resolution table (lux/count)
        let resolution = match (gain, integration_time) {
            // Gain 1x
            (0, 0) => 0.1152,  // IT=25ms
            (0, 1) => 0.0576,  // IT=50ms  
            (0, 2) => 0.0288,  // IT=100ms
            (0, 3) => 0.0144,  // IT=200ms
            (0, 4) => 0.0072,  // IT=400ms
            (0, 5) => 0.0036,  // IT=800ms
            // Gain 2x
            (1, 0) => 0.0576,  // IT=25ms
            (1, 1) => 0.0288,  // IT=50ms
            (1, 2) => 0.0144,  // IT=100ms
            (1, 3) => 0.0072,  // IT=200ms
            (1, 4) => 0.0036,  // IT=400ms
            (1, 5) => 0.0018,  // IT=800ms
            // Gain 1/8x
            (2, 0) => 0.9216,  // IT=25ms
            (2, 1) => 0.4608,  // IT=50ms
            (2, 2) => 0.2304,  // IT=100ms
            (2, 3) => 0.1152,  // IT=200ms
            (2, 4) => 0.0576,  // IT=400ms
            (2, 5) => 0.0288,  // IT=800ms
            // Gain 1/4x
            (3, 0) => 0.4608,  // IT=25ms
            (3, 1) => 0.2304,  // IT=50ms
            (3, 2) => 0.1152,  // IT=100ms
            (3, 3) => 0.0576,  // IT=200ms
            (3, 4) => 0.0288,  // IT=400ms
            (3, 5) => 0.0144,  // IT=800ms
            _ => 0.0288, // Default fallback
        };
        
        let lux = als_raw as f32 * resolution;
        info!("ALS raw: {}, Gain: {}, IT: {}, Resolution: {:.4}, Lux: {:.2}", 
              als_raw, gain, integration_time, resolution, lux);
        Ok(lux)
    }

    #[allow(dead_code)]
    pub fn power_down(&mut self) -> Result<(), io::Error> {
        // Set ALS shutdown bit
        let mut config = self.read_register(ALS_CONF_0)?;
        config |= 0x01; // Set shutdown bit
        self.write_register(ALS_CONF_0, config)
    }

    #[allow(dead_code)]
    pub fn power_up(&mut self) -> Result<(), io::Error> {
        // Clear ALS shutdown bit
        let mut config = self.read_register(ALS_CONF_0)?;
        config &= 0xFFFE; // Clear shutdown bit
        self.write_register(ALS_CONF_0, config)
    }

    fn read_register(&mut self, reg: u8) -> Result<u16, io::Error> {
        let mut buffer = [0u8; 2];
        
        // Retry logic for I2C communication
        for attempt in 1..=3 {
            match self.i2c.write_read(self.address, &[reg], &mut buffer) {
                Ok(_) => {
                    // VEML7700 returns little-endian 16-bit values
                    let value = u16::from_le_bytes(buffer);
                    return Ok(value);
                },
                Err(e) => {
                    if attempt == 3 {
                        error!("Failed to read register 0x{:02X} after {} attempts: {:?}", reg, attempt, e);
                        return Err(io::Error::new(io::ErrorKind::Other, format!("I2C read failed: {:?}", e)));
                    }
                    debug!("Read attempt {} failed, retrying: {:?}", attempt, e);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        unreachable!()
    }

    fn write_register(&mut self, reg: u8, value: u16) -> Result<(), io::Error> {
        let bytes = value.to_le_bytes(); // VEML7700 expects little-endian
        let data = [reg, bytes[0], bytes[1]];
        
        // Retry logic for I2C communication
        for attempt in 1..=3 {
            match self.i2c.write(self.address, &data) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt == 3 {
                        error!("Failed to write register 0x{:02X} after {} attempts: {:?}", reg, attempt, e);
                        return Err(io::Error::new(io::ErrorKind::Other, format!("I2C write failed: {:?}", e)));
                    }
                    debug!("Write attempt {} failed, retrying: {:?}", attempt, e);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        unreachable!()
    }
}
