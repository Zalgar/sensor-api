# Sensor API Project

This project contains Rust-based APIs that interface directly with sensors and expose them via HTTP endpoints. This approach was developed to standardize sensor communication - instead of requiring bespoke code for each sensor, everything is accessible through plain HTTP APIs.

## Overview

The project consists of individual sensor APIs and a centralized sensor-collector service that aggregates data from all sensors and forwards it to InfluxDB and MQTT brokers.

### Current Sensors

| Sensor    | Port | Status    | Measurements |
|-----------|------|-----------|--------------|
| BME280    | 5000 | ✅ Active | Temperature, Humidity, Pressure |
| PMSA003I  | 5001 | ✅ Active | Particulate Matter (PM1.0, PM2.5, PM10) |
| SCD-41    | 5002 | ✅ Active | CO2, Temperature, Humidity |
| LTR390    | 5003 | ✅ Active | UV Index, Ambient Light |
| BH1750    | 5004 | ✅ Active | Ambient Light (Lux) |
| VEML7700  | 5005 | ✅ Active | Ambient Light (Advanced, Configurable) |
| TSL2591   | 5006 | ✅ Active | Ambient Light (High Precision, Configurable) |

### Sensor-Collector Service
- **Port**: 8080
- **Purpose**: Aggregates data from all sensor APIs
- **Outputs**: InfluxDB, MQTT

## Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Sensor APIs │    │ Collector   │    │ Data Stores │
│ (5000-5006) │───▶│ Service     │───▶│ InfluxDB    │
│             │    │ (8080)      │    │ MQTT        │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Setup and Installation

### Prerequisites

#### System Requirements
- Rust toolchain with cross-compilation support
- Access to I2C bus for sensor communication
- systemd for service management (production)

#### Install Dependencies
```bash
# System packages
sudo apt-get install pkg-config libssl-dev musl-tools gcc-arm-linux-gnueabihf

# Rust cross-compilation target
rustup target add arm-unknown-linux-musleabihf
```

### I2C Configuration

1. **Find I2C group**:
   ```bash
   ls -l /dev/i2c*
   ```

2. **Add sensor user to I2C group**:
   ```bash
   sudo usermod -a -G i2c sensor
   ```

3. **Set permissions**:
   ```bash
   sudo chmod 660 /dev/i2c-1
   ```

### Building

#### Individual Sensor APIs
```bash
# Navigate to sensor directory
cd bme280  # or any sensor directory

# Build for ARM Linux (production)
cargo build --release --target arm-unknown-linux-musleabihf

# Build for local development
cargo build --release
```

#### All Sensors
```bash
# Build script for all sensors (create if needed)
for sensor in bme280 pmsa003i scd-41 ltr390 bh1750 veml7700 tsl2591; do
    echo "Building $sensor..."
    cd $sensor
    cargo build --release --target arm-unknown-linux-musleabihf
    cd ..
done
```

### Deployment

#### SystemD Services
All sensors include systemd service files in the `systemd/` directory:

```bash
# Copy service files
sudo cp systemd/*.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable and start services
sudo systemctl enable bme280.service pmsa003i.service scd-41.service
sudo systemctl enable ltr390.service bh1750.service veml7700.service tsl2591.service
sudo systemctl enable sensor-collector.service

# Start all services
sudo systemctl start bme280.service pmsa003i.service scd-41.service
sudo systemctl start ltr390.service bh1750.service veml7700.service tsl2591.service
sudo systemctl start sensor-collector.service
```

#### File Structure (Production)
```
/srv/
├── bme280/
│   ├── bme280_api          # Binary
│   └── config.json         # Configuration
├── pmsa003i/
├── scd-41/
├── ltr390/
├── bh1750/
├── veml7700/
├── tsl2591/
└── sensor-collector/
    ├── sensor-collector    # Binary
    └── config.json         # Configuration
```

## Configuration

Each sensor API uses a JSON configuration file that is automatically created with default values on first run.

### Standard Configuration Structure
```json
{
  "network_port": 5000,
  "bind_address": "0.0.0.0",
  "i2c_bus_device_path": "/dev/i2c-1",
  "i2c_address_decimal": 118
}
```

### Sensor-Specific Configurations

#### TSL2591 (Advanced Light Sensor)
```json
{
  "network_port": 5006,
  "bind_address": "0.0.0.0", 
  "i2c_bus_device_path": "/dev/i2c-1",
  "i2c_address_decimal": 41,
  "gain": 1,
  "integration_time": 1
}
```

#### VEML7700 (Configurable Light Sensor)
```json
{
  "network_port": 5005,
  "bind_address": "0.0.0.0",
  "i2c_bus_device_path": "/dev/i2c-1", 
  "i2c_address_decimal": 16,
  "als_gain": 0,
  "integration_time": 2
}
```

## API Reference

### Common Endpoint
- **GET** `/sensor_data` - Returns current sensor readings

### Response Format
All APIs return standardized JSON responses:

```json
{
  "timestamp": "2025-08-18T10:30:45.123Z",
  "model": "BME280",
  // sensor-specific data fields...
}
```

### Example Responses

#### BME280 (Temperature/Humidity/Pressure)
```json
{
  "timestamp": "2025-08-18T10:30:45.123Z",
  "model": "BME280",
  "temperature": 25.0,
  "humidity": 40.0,
  "pressure": 1013.25,
  "altitude": 100.0
}
```

#### LTR390 (UV/Light)
```json
{
  "timestamp": "2025-08-18T10:30:45.123Z",
  "model": "LTR390",
  "uv_index": 2.5,
  "light_ambient": 150.0
}
```

#### TSL2591 (Precision Light)
```json
{
  "timestamp": "2025-08-18T10:30:45.123Z",
  "model": "TSL2591",
  "light_ambient": 245.67,
  "light_infrared": 89.12,
  "light_visible": 156.55,
  "light_full_spectrum": 245.67
}
```

## Development

### Project Structure
```
sensor-api/
├── README.md
├── DESIGN_GUIDELINES.md      # Development standards
├── PROJECT_IMPROVEMENTS.md   # Change log
├── systemd/                  # Service files
├── bme280/                   # Individual sensor APIs
├── pmsa003i/
├── scd-41/
├── ltr390/
├── bh1750/
├── veml7700/
├── tsl2591/
└── sensor-collector/         # Data aggregation service
```

### Design Guidelines
See [DESIGN_GUIDELINES.md](DESIGN_GUIDELINES.md) for:
- Port assignment strategy
- Configuration standards
- Code organization patterns
- SystemD service templates
- API response formats

### Adding New Sensors
1. Follow port assignment strategy (5007, 5008, etc.)
2. Use standard configuration structure
3. Implement consistent error handling with `log::error!`
4. Create SystemD service file
5. Add to sensor-collector configuration
6. Update this README

## Monitoring and Troubleshooting

### Service Status
```bash
# Check individual sensor status
sudo systemctl status bme280.service

# Check all sensor services
sudo systemctl status bme280 pmsa003i scd-41 ltr390 bh1750 veml7700 tsl2591

# Check sensor-collector
sudo systemctl status sensor-collector
```

### Logs
```bash
# View sensor logs
sudo journalctl -u bme280.service -f

# View collector logs  
sudo journalctl -u sensor-collector.service -f
```

### Testing APIs
```bash
# Test individual sensor
curl http://localhost:5000/sensor_data

# Test all sensors
for port in {5000..5006}; do
  echo "Testing port $port:"
  curl http://localhost:$port/sensor_data
  echo -e "\n"
done
```

## Contributing

1. Follow design guidelines in [DESIGN_GUIDELINES.md](DESIGN_GUIDELINES.md)
2. Ensure proper error handling with structured logging
3. Add configuration documentation
4. Include SystemD service file
5. Update this README with new sensor information