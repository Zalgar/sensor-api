# Rust Sensor Project

This project is designed to query sensor data from a JSON API and log the results to InfluxDB. It dynamically determines the available data from the sensor API and manages configuration settings through a self-creating configuration file.

## Project Structure

```
rust-sensor-project
├── src
│   ├── main.rs        # Entry point of the application
│   ├── api.rs         # Functions for querying the sensor API
│   ├── config.rs      # Configuration settings management
│   ├── influxdb.rs    # InfluxDB connection and logging
│   └── utils.rs       # Utility functions
├── Cargo.toml         # Rust project configuration
└── README.md          # Project documentation
```

## Setup Instructions

1. **Clone the repository:**
   ```
   git clone <repository-url>
   cd rust-sensor-project
   ```

2. **Install Rust:**
   Follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install) to install Rust.

3. **Build the project:**
   ```
   cargo build
   ```

4. **Run the application:**
   ```
   cargo run
   ```

## Configuration

The application will automatically create a configuration file if it does not exist. The configuration file contains settings for the API endpoint and InfluxDB connection details.

## Usage

Once the application is running, it will periodically query the sensor API at `http://172.20.20.9/sensor_data`, retrieve the available data, and log it to InfluxDB.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for details.