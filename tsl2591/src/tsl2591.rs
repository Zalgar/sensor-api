use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use log::{info, error, debug};
use std::io;
use std::thread;
use std::time::Duration;

// TSL2591 register addresses
const ENABLE: u8 = 0x00;
const CONTROL: u8 = 0x01;
const C0DATAL: u8 = 0x14;
const C0DATAH: u8 = 0x15;
const C1DATAL: u8 = 0x16;
const C1DATAH: u8 = 0x17;

// Commands
const COMMAND_BIT: u8 = 0x80;

// Enable register values
const ENABLE_POWERON: u8 = 0x01;
const ENABLE_AEN: u8 = 0x02;

// Gain values
const GAIN_LOW: u8 = 0x00;   // 1x gain
const GAIN_MED: u8 = 0x10;   // 25x gain
const GAIN_HIGH: u8 = 0x20;  // 428x gain
const GAIN_MAX: u8 = 0x30;   // 9876x gain

// Integration time values
const INTEGRATIONTIME_100MS: u8 = 0x00;
const INTEGRATIONTIME_200MS: u8 = 0x01;
const INTEGRATIONTIME_300MS: u8 = 0x02;
const INTEGRATIONTIME_400MS: u8 = 0x03;
const INTEGRATIONTIME_500MS: u8 = 0x04;
const INTEGRATIONTIME_600MS: u8 = 0x05;

pub struct TSL2591 {
    i2c: I2cdev,
    address: u8,
    gain: u8,
    integration_time: u8,
}

impl TSL2591 {
    pub fn new(i2c: I2cdev, address: u8) -> Result<Self, io::Error> {
        Ok(TSL2591 {
            i2c,
            address,
            gain: GAIN_MED,
            integration_time: INTEGRATIONTIME_100MS,
        })
    }

    pub fn init(&mut self, gain: u16, integration_time: u16) -> Result<(), io::Error> {
        // Enable the device
        self.write_register(ENABLE, ENABLE_POWERON | ENABLE_AEN)?;
        
        // Set gain
        self.gain = match gain {
            0 => GAIN_LOW,
            1 => GAIN_MED,
            2 => GAIN_HIGH,
            3 => GAIN_MAX,
            _ => GAIN_MED,
        };

        // Set integration time
        self.integration_time = match integration_time {
            0 => INTEGRATIONTIME_100MS,
            1 => INTEGRATIONTIME_200MS,
            2 => INTEGRATIONTIME_300MS,
            3 => INTEGRATIONTIME_400MS,
            4 => INTEGRATIONTIME_500MS,
            5 => INTEGRATIONTIME_600MS,
            _ => INTEGRATIONTIME_100MS,
        };

        // Write control register with gain and integration time
        let control_value = self.gain | self.integration_time;
        self.write_register(CONTROL, control_value)?;

        thread::sleep(Duration::from_millis(120));
        
        info!("TSL2591 sensor initialized successfully");
        Ok(())
    }

    pub fn read_lux(&mut self) -> Result<(u16, u16, f32), io::Error> {
        // Read channel 0 (visible + IR)
        let c0_low = self.read_register(C0DATAL)?;
        let c0_high = self.read_register(C0DATAH)?;
        let channel0 = ((c0_high as u16) << 8) | (c0_low as u16);

        // Read channel 1 (IR only)
        let c1_low = self.read_register(C1DATAL)?;
        let c1_high = self.read_register(C1DATAH)?;
        let channel1 = ((c1_high as u16) << 8) | (c1_low as u16);

        // Calculate visible light (channel0 - channel1)
        let visible = channel0.saturating_sub(channel1);

        // Calculate lux based on gain and integration time
        let lux = self.calculate_lux(channel0, channel1);

        debug!("TSL2591 readings - Visible: {}, IR: {}, Lux: {:.2}", visible, channel1, lux);
        
        Ok((visible, channel1, lux))
    }

    fn calculate_lux(&self, channel0: u16, channel1: u16) -> f32 {
        if channel0 == 0 {
            return 0.0;
        }

        // Get gain multiplier
        let gain_factor = match self.gain {
            GAIN_LOW => 1.0,
            GAIN_MED => 25.0,
            GAIN_HIGH => 428.0,
            GAIN_MAX => 9876.0,
            _ => 25.0,
        };

        // Get integration time factor (in seconds)
        let time_factor = match self.integration_time {
            INTEGRATIONTIME_100MS => 0.1,
            INTEGRATIONTIME_200MS => 0.2,
            INTEGRATIONTIME_300MS => 0.3,
            INTEGRATIONTIME_400MS => 0.4,
            INTEGRATIONTIME_500MS => 0.5,
            INTEGRATIONTIME_600MS => 0.6,
            _ => 0.1,
        };

        // Calculate CPL (Counts Per Lux)
        let cpl = (time_factor * gain_factor) / 408.0;
        
        // Calculate ratio
        let ratio = (channel1 as f32) / (channel0 as f32);
        
        // Calculate lux based on ratio
        let lux = if ratio <= 0.5 {
            (0.0315 * channel0 as f32) - (0.0593 * channel0 as f32 * ratio.powf(1.4))
        } else if ratio <= 0.61 {
            (0.0229 * channel0 as f32) - (0.0291 * channel1 as f32)
        } else if ratio <= 0.80 {
            (0.0157 * channel0 as f32) - (0.0180 * channel1 as f32)
        } else if ratio <= 1.30 {
            (0.00338 * channel0 as f32) - (0.00260 * channel1 as f32)
        } else {
            0.0
        };

        lux / cpl
    }

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), io::Error> {
        let cmd = COMMAND_BIT | register;
        match self.i2c.write(self.address, &[cmd, value]) {
            Ok(_) => {
                debug!("TSL2591 write register 0x{:02X} = 0x{:02X}", register, value);
                Ok(())
            }
            Err(e) => {
                error!("TSL2591 failed to write register 0x{:02X}: {}", register, e);
                Err(io::Error::new(io::ErrorKind::Other, format!("I2C write error: {}", e)))
            }
        }
    }

    fn read_register(&mut self, register: u8) -> Result<u8, io::Error> {
        let cmd = COMMAND_BIT | register;
        let mut buffer = [0u8; 1];
        
        match self.i2c.write_read(self.address, &[cmd], &mut buffer) {
            Ok(_) => {
                debug!("TSL2591 read register 0x{:02X} = 0x{:02X}", register, buffer[0]);
                Ok(buffer[0])
            }
            Err(e) => {
                error!("TSL2591 failed to read register 0x{:02X}: {}", register, e);
                Err(io::Error::new(io::ErrorKind::Other, format!("I2C read error: {}", e)))
            }
        }
    }
}
