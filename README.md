# Sensor API

This project contains two Rust applications for reading sensor data from BME280 and PMSA003I sensors using the I2C bus and exposing the data through a web API using Actix-web.

## Project Structure

```
.gitignore
bme280/
    Cargo.lock
    Cargo.toml
    src/
        main.rs
    target/
pmsa003i/
    Cargo.lock
    Cargo.toml
    src/
        main.rs
    target/
README.md
```

## BME280 Sensor Application

The BME280 application reads temperature, humidity, pressure, and altitude data from a BME280 sensor.

### How to Run

1. Navigate to the 

bme280

 directory:
    ```sh
    cd bme280
    ```

2. Build the project:
    ```sh
    cargo build
    ```

3. Run the project:
    ```sh
    cargo run
    ```

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

## PMSA003I Sensor Application

The PMSA003I application reads particulate matter concentrations (PM1.0, PM2.5, PM10) from a PMSA003I sensor.

### How to Run

1. Navigate to the 

pmsa003i

 directory:
    ```sh
    cd pmsa003i
    ```

2. Build the project:
    ```sh
    cargo build
    ```

3. Run the project:
    ```sh
    cargo run
    ```

### API Endpoint

- **GET /sensor_data**: Returns the sensor data in JSON format.

### Example Response

```json
{
    "timestamp": "2023-10-01T12:00:00Z",
    "model": "PMSA003I",
    "pm1_0": 10,
    "pm2_5": 20,
    "pm10": 30
}
```

## Common Dependencies

Both applications use the following common dependencies:
- `actix-web`: For building the web server and handling HTTP requests.
- 

linux_embedded_hal

: For hardware abstraction layer (HAL) implementations.
- 

serde

: For serializing and deserializing data.
- 

chrono

: For handling date and time.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
