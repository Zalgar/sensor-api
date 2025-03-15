# Sensor API

This project contains two Rust applications for reading sensor data from BME280 and PMSA003I sensors using the I2C bus and exposing the data through a web API using Actix-web.

# Base Config
Ensure you give the sensor user if using systemd units provided has access to the i2c bus(s)

Find the group which provides access
ls -l /dev/i2c*

Most cases its the group i2c
sudo usermod -a -G i2c sensor

You may also need to change perms for the path too, this might need to be refined.
sudo chmod 660 /dev/i2c-1

### How to build

1. Navigate to the sensor directory:
    ```sh
    cd "sensor"
    ```

2. Build the project:
    ```sh
    cargo build
    ```

3. Run the project:
    ```sh
    cargo run
    ```
4. You can get a binary from target/

If you are not cross compiling for Arm, you'll need to remove the .cargo/cargo.toml which specifies the arch. If you are building on the device locally this is not required.

### API Endpoint

- **GET /sensor_data**: Returns the sensor data in JSON format.

### Example Response

```json
{
    "timestamp": "2023-10-01T12:00:00Z",
    "model": "BME280",
    "temperature": 25.0,
    "humidity": 40.0,
    "pressure": 1013.25,
    "altitude": 100.0
}
```
