use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use log::{info, error, debug};
use std::io;
use std::thread;
use std::time::Duration;

pub struct LTR390 {
    i2c: I2cdev,
    address: u8,
}

impl LTR390 {
    pub fn new(i2c: I2cdev, address: u8) -> Result<Self, io::Error> {
        Ok(LTR390 { i2c, address })
    }

    pub fn begin(&mut self) -> Result<bool, io::Error> {
        info!("Initializing LTR390 sensor");

        // Check part ID
        let part_id = self.read_register(0x06)?; // LTR390_PART_ID
        info!("Part ID: {:#X}", part_id);
        if (part_id >> 4) != 0xB {
            error!("Invalid Part ID: {:#X}", part_id);
            return Ok(false);
        }

        // Attempt soft reset (but don't fail initialization if reset fails)
        match self.reset() {
            Ok(true) => info!("Reset completed successfully"),
            Ok(false) => {
                debug!("Reset failed, but continuing with initialization");
                // Continue anyway - some sensors work without reset
            }
            Err(e) => {
                debug!("Reset error: {:?}, continuing with initialization", e);
                // Continue anyway - reset might not be critical
            }
        }

        // Configure the sensor for ALS (Ambient Light Sensor) mode first
        info!("Configuring sensor for ALS mode");
        self.set_mode(false)?; // false = ALS mode, true = UVS mode
        
        // Set gain and integration time for better sensitivity
        self.set_gain(0x01)?; // Gain = 3x
        self.set_integration_time(0x02)?; // Integration time = 100ms
        
        // Enable the sensor with retry logic
        let mut enable_success = false;
        for attempt in 1..=3 {
            match self.enable(true) {
                Ok(_) => {
                    thread::sleep(Duration::from_millis(50));
                    match self.enabled() {
                        Ok(true) => {
                            enable_success = true;
                            break;
                        }
                        Ok(false) => {
                            error!("Enable attempt {}: sensor not enabled after command", attempt);
                        }
                        Err(e) => {
                            error!("Enable attempt {}: failed to check enabled status: {:?}", attempt, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Enable attempt {}: failed to send enable command: {:?}", attempt, e);
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        if !enable_success {
            error!("Failed to enable the sensor after multiple attempts");
            return Ok(false);
        }

        // Wait for first measurement
        thread::sleep(Duration::from_millis(200));

        info!("LTR390 sensor initialized successfully");
        Ok(true)
    }

    pub fn reset(&mut self) -> Result<bool, io::Error> {
        info!("Performing soft reset");
        
        // Try to write the reset command with retry
        let mut retry_count = 0;
        const MAX_RETRIES: u8 = 3;
        
        while retry_count < MAX_RETRIES {
            match self.write_register(0x00, 0x10) {
                Ok(_) => break,
                Err(e) => {
                    retry_count += 1;
                    debug!("Reset write attempt {} failed: {:?}", retry_count, e);
                    if retry_count >= MAX_RETRIES {
                        return Err(e);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        
        thread::sleep(Duration::from_millis(100)); // Longer wait for reset to complete

        // Check if reset bit is cleared (with retry)
        let mut reset_success = false;
        for attempt in 1..=5 {
            match self.read_register(0x00) {
                Ok(reg_value) => {
                    let reset_bit = reg_value & 0x10;
                    if reset_bit == 0 {
                        reset_success = true;
                        break;
                    } else {
                        info!("Reset attempt {}: reset bit still set (0x{:02X})", attempt, reg_value);
                        thread::sleep(Duration::from_millis(50));
                    }
                }
                Err(e) => {
                    error!("Failed to read reset status on attempt {}: {:?}", attempt, e);
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        
        if reset_success {
            info!("Soft reset successful");
        } else {
            debug!("Soft reset failed after multiple attempts");
        }
        Ok(reset_success)
    }

    pub fn read_als(&mut self) -> Result<u32, io::Error> {
        // Set to ALS mode
        self.set_mode(false)?;
        
        // Wait for mode switch and measurement
        thread::sleep(Duration::from_millis(200));
        
        // Wait for ALS data to be ready
        let mut attempts = 0;
        loop {
            let status = self.read_register(0x07)?; // LTR390_MAIN_STATUS
            
            // Check if ALS data is ready (bit 3 for new data)
            if (status & 0x08) != 0 {
                info!("ALS data ready, status: 0x{:02X}", status);
                break;
            }
            
            attempts += 1;
            if attempts >= 10 { // Wait up to 1 second
                info!("ALS data timeout, status: 0x{:02X}", status);
                break;
            }
            
            thread::sleep(Duration::from_millis(100));
        }
        
        let data = self.read_data_register(0x0D, 3)?; // LTR390_ALSDATA
        info!("ALS raw data: {}", data);
        Ok(data)
    }

    pub fn read_uvs(&mut self) -> Result<u32, io::Error> {
        // Set to UVS mode  
        self.set_mode(true)?;
        
        // Wait longer for mode switch and first measurement
        thread::sleep(Duration::from_millis(300));
        
        // Wait for fresh UVS data - keep checking until we get new data
        let mut attempts = 0;
        let mut previous_data = 0u32;
        let mut current_data;
        
        loop {
            let status = self.read_register(0x07)?; // LTR390_MAIN_STATUS
            
            // Read current data
            current_data = self.read_data_register(0x10, 3)?; // LTR390_UVSDATA
            
            // Check if we have new data (either status indicates ready or data changed)
            if (status & 0x08) != 0 || (status & 0x10) != 0 || current_data != previous_data {
                info!("UVS data ready, status: 0x{:02X}", status);
                break;
            }
            
            previous_data = current_data;
            attempts += 1;
            if attempts >= 20 { // Wait up to 2 seconds
                info!("UVS data timeout, status: 0x{:02X}, using current data", status);
                break;
            }
            
            thread::sleep(Duration::from_millis(100));
        }
        
        info!("UVS raw data: {}", current_data);
        Ok(current_data)
    }

    pub fn enable(&mut self, en: bool) -> Result<(), io::Error> {
        let mut ctrl = self.read_register(0x00)?; // LTR390_MAIN_CTRL
        if en {
            ctrl |= 0x02;
        } else {
            ctrl &= !0x02;
        }
        self.write_register(0x00, ctrl) // LTR390_MAIN_CTRL
    }

    pub fn enabled(&mut self) -> Result<bool, io::Error> {
        let ctrl = self.read_register(0x00)?; // LTR390_MAIN_CTRL
        Ok((ctrl & 0x02) != 0)
    }

    pub fn set_mode(&mut self, uv_mode: bool) -> Result<(), io::Error> {
        let mut ctrl = self.read_register(0x00)?; // LTR390_MAIN_CTRL
        if uv_mode {
            ctrl |= 0x08; // UVS mode
        } else {
            ctrl &= !0x08; // ALS mode
        }
        self.write_register(0x00, ctrl)
    }

    pub fn set_gain(&mut self, gain: u8) -> Result<(), io::Error> {
        // LTR390_ALS_UVS_GAIN register (0x05)
        // Gain values: 0=1x, 1=3x, 2=6x, 3=9x, 4=18x
        self.write_register(0x05, gain & 0x07)
    }

    pub fn set_integration_time(&mut self, time: u8) -> Result<(), io::Error> {
        // LTR390_ALS_UVS_MEAS_RATE register (0x04)
        // Integration time in bits 0-2, measurement rate in bits 4-6
        let mut meas_rate = self.read_register(0x04).unwrap_or(0x22); // Default: 500ms rate
        meas_rate = (meas_rate & 0xF8) | (time & 0x07); // Keep rate, set integration time
        self.write_register(0x04, meas_rate)
    }

    fn read_register(&mut self, reg: u8) -> Result<u8, io::Error> {
        let mut buf = [0u8; 1];
        
        // Retry logic for I2C communication
        for attempt in 1..=3 {
            match self.i2c.write_read(self.address, &[reg], &mut buf) {
                Ok(_) => return Ok(buf[0]),
                Err(e) => {
                    if attempt == 3 {
                        return Err(io::Error::new(io::ErrorKind::Other, format!("I2C read failed after {} attempts: {:?}", attempt, e)));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        unreachable!()
    }

    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), io::Error> {
        // Retry logic for I2C communication
        for attempt in 1..=3 {
            match self.i2c.write(self.address, &[reg, value]) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt == 3 {
                        return Err(io::Error::new(io::ErrorKind::Other, format!("I2C write failed after {} attempts: {:?}", attempt, e)));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        unreachable!()
    }

    fn read_data_register(&mut self, reg: u8, len: usize) -> Result<u32, io::Error> {
        let mut value = 0u32;
        for i in 0..len {
            let byte = self.read_register(reg + i as u8)?;
            value |= (byte as u32) << (8 * i);
        }
        Ok(value)
    }
}