use embedded_hal::i2c::I2c;
use log::{info, error};
use std::fmt::Debug;

pub struct LTR390<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> LTR390<I2C>
where
    I2C: I2c<Error = E>,
    E: Debug,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        LTR390 { i2c, address }
    }

    pub fn begin(&mut self) -> Result<bool, E> {
        info!("Initializing LTR390 sensor");

        // Check part ID
        let part_id = self.read_register(0x06)?; // LTR390_PART_ID
        info!("Part ID: {:#X}", part_id);
        if (part_id >> 4) != 0xB {
            error!("Invalid Part ID: {:#X}", part_id);
            return Ok(false);
        }

        // Perform a soft reset
        if !self.reset()? {
            error!("Failed to reset the sensor");
            return Ok(false);
        }

        // Enable the sensor
        self.enable(true)?;
        if !self.enabled()? {
            error!("Failed to enable the sensor");
            return Ok(false);
        }

        info!("LTR390 sensor initialized successfully");
        Ok(true)
    }

    pub fn reset(&mut self) -> Result<bool, E> {
        info!("Performing soft reset");
        self.write_register(0x00, 0x10)?; // LTR390_MAIN_CTRL, soft reset bit
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Re-initialize I2C after reset
        info!("Re-initializing I2C after soft reset");
        if let Err(e) = self.i2c.write(self.address, &[0x00, 0x10]) {
            error!("Failed to re-initialize I2C: {:?}", e);
            return Err(e);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Retry reading the reset bit
        let reset_bit = self.read_register(0x00)? & 0x10; // LTR390_MAIN_CTRL
        let success = reset_bit == 0;
        if success {
            info!("Soft reset successful");
        } else {
            error!("Soft reset failed");
        }
        Ok(success)
    }

    pub fn read_als(&mut self) -> Result<u32, E> {
        self.read_data_register(0x0D, 3) // LTR390_ALSDATA
    }

    pub fn read_uvs(&mut self) -> Result<u32, E> {
        self.read_data_register(0x10, 3) // LTR390_UVSDATA
    }

    pub fn enable(&mut self, en: bool) -> Result<(), E> {
        let mut ctrl = self.read_register(0x00)?; // LTR390_MAIN_CTRL
        if en {
            ctrl |= 0x02;
        } else {
            ctrl &= !0x02;
        }
        self.write_register(0x00, ctrl) // LTR390_MAIN_CTRL
    }

    pub fn enabled(&mut self) -> Result<bool, E> {
        let ctrl = self.read_register(0x00)?; // LTR390_MAIN_CTRL
        Ok((ctrl & 0x02) != 0)
    }

    fn read_register(&mut self, reg: u8) -> Result<u8, E> {
        let mut buf = [0];
        self.i2c.write_read(self.address, &[reg], &mut buf)?;
        Ok(buf[0])
    }

    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[reg, value])
    }

    fn read_data_register(&mut self, reg: u8, len: usize) -> Result<u32, E> {
        let mut buf = vec![0; len];
        self.i2c.write_read(self.address, &[reg], &mut buf)?;
        let mut value = 0;
        for (i, &b) in buf.iter().enumerate() {
            value |= (b as u32) << (8 * i);
        }
        Ok(value)
    }
}