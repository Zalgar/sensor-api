# Sensor API Project Design Guidelines

## Overview
This document defines the design patterns, conventions, and standards for the Sensor API project to ensure consistency across all sensor implementations.

## 1. Port Assignment Strategy

### Current Port Allocation
| Sensor    | Port | Status    | Notes |
|-----------|------|-----------|-------|
| BME280    | 5000 | ✅ Active | Temperature, Humidity, Pressure |
| PMSA003I  | 5001 | ✅ Active | Particulate Matter |
| SCD-41    | 5002 | ✅ Active | CO2, Temperature, Humidity |
| LTR390    | 5003 | ✅ Active | UV and Ambient Light |
| BH1750    | 5004 | ✅ Active | Ambient Light |
| VEML7700  | 5005 | ✅ Active | Ambient Light (Advanced) |
| TSL2591   | 5006 | ✅ Active | Ambient Light (Precision) |

### Port Assignment Rules
- **Base Range**: 5000-5099 reserved for sensor APIs
- **Sequential Assignment**: Assign ports sequentially starting from 5000
- **Collector Service**: Port 8080 (sensor-collector)
- **Reserved Ranges**: 
  - 5000-5049: Individual sensor APIs
  - 5050-5099: Future sensor APIs or specialized services

## 2. Configuration Structure

### Standard Config Fields
All sensor APIs MUST include these standard configuration fields:

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    // REQUIRED: Network configuration
    network_port: u16,
    bind_address: String,
    
    // REQUIRED: I2C configuration  
    i2c_bus_device_path: String,
    
    // CONDITIONAL: I2C address (if sensor uses I2C)
    i2c_address_decimal: u16,
    
    // OPTIONAL: Sensor-specific configuration parameters
    // (sensor-specific fields go here)
}
```

### Standard Default Values
```rust
impl Default for Config {
    fn default() -> Self {
        Config {
            network_port: 50XX, // See port assignment table
            bind_address: String::from("0.0.0.0"),
            i2c_bus_device_path: String::from("/dev/i2c-1"),
            i2c_address_decimal: 0xXX, // Sensor-specific default
            // sensor-specific defaults...
        }
    }
}
```

### Configuration File Naming
- **Filename**: `config.json`
- **Location**: Root of sensor project directory
- **Format**: JSON with pretty printing
- **Auto-generation**: If config doesn't exist, create with defaults

## 3. Project Structure Standards

### Directory Layout
```
sensor-name/
├── Cargo.toml              # Project manifest
├── config.json             # Configuration file
├── src/
│   ├── main.rs             # Main application entry point
│   └── sensor-name.rs      # Sensor-specific driver module
└── target/                 # Build artifacts
```

### File Naming Conventions
- **Project Name**: `{sensor_name}_api` (e.g., `bme280_api`)
- **Binary Name**: `{sensor_name}_api` (matches project name)
- **Module Name**: `{sensor_name}.rs` (e.g., `bme280.rs`)
- **SystemD Service**: `{sensor_name}.service`

## 4. Cargo.toml Standards

### Required Configuration
```toml
[package]
name = "{sensor_name}_api"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{sensor_name}_api"
path = "src/main.rs"

[dependencies]
actix-web = "4.0.0-beta.7"
linux-embedded-hal = "0.4"
embedded-hal = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
env_logger = "0.10"
log = "0.4"
# sensor-specific dependencies...
```

### Cross-Compilation Target
- **Primary Target**: `arm-unknown-linux-musleabihf`
- **Development Target**: `x86_64-pc-windows-msvc` (Windows dev environment)

## 5. API Response Standards

### Standard Response Structure
```rust
#[derive(Serialize)]
struct SensorData {
    timestamp: String,        // RFC3339 format
    model: String,           // Sensor model name
    // sensor-specific fields...
}
```

### Timestamp Format
- **Standard**: RFC3339 format using `chrono::Utc::now().to_rfc3339()`
- **Example**: `"2025-08-18T10:30:45.123Z"`

### HTTP Endpoints
- **Primary Endpoint**: `GET /sensor_data`
- **Response Format**: JSON
- **Content-Type**: `application/json`

## 6. Error Handling Standards

### HTTP Status Codes
- **200 OK**: Successful sensor reading
- **500 Internal Server Error**: Sensor communication failure, I2C errors, initialization errors

### Error Response Format
```rust
// Return plain text error messages for 500 errors
HttpResponse::InternalServerError().body("Failed to read sensor data")
```

### Logging Standards
- **Framework**: `log` crate with `env_logger`
- **Default Level**: `info`
- **Error Logging**: Use `log::error!` for failures
- **Debug Logging**: Use `log::debug!` for detailed tracing
- **Info Logging**: Use `log::info!` for lifecycle events

## 7. SystemD Service Standards

### Service File Template
```ini
[Unit]
Description={Sensor Name} Sensor API
After=network.target
Wants=network-online.target

[Service]
Restart=always
Type=simple
ExecStart=/srv/{sensor_name}/{sensor_name}_api
WorkingDirectory=/srv/{sensor_name}
User=sensor

[Install]
WantedBy=multi-user.target
```

### Service Naming
- **Pattern**: `{sensor_name}.service`
- **Location**: `systemd/` directory in project root

## 8. Code Organization Standards

### Import Organization
```rust
// External crates
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::fs;
use std::fs::File;
use std::io::Write;
use env_logger::Env;
use log::{info, error, debug};

// Local modules
mod {sensor_name};
use {sensor_name}::{SensorStruct};
```

### Function Organization
1. Configuration structures and implementations
2. Helper functions (`read_or_create_config`)
3. Sensor data structures
4. API handlers (`get_sensor_data`)
5. Main function

### Sensor Module Standards
```rust
// sensor_name.rs
use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use log::{info, error, debug};
use std::io;
use std::thread;
use std::time::Duration;

pub struct SensorName {
    i2c: I2cdev,
    address: u8,
    // sensor-specific fields...
}

impl SensorName {
    pub fn new(i2c: I2cdev, address: u8) -> Result<Self, io::Error> {
        // Constructor logic
    }
    
    pub fn init(&mut self, /* config params */) -> Result<(), io::Error> {
        // Initialization logic
    }
    
    pub fn read_data(&mut self) -> Result<(/* return types */), io::Error> {
        // Data reading logic
    }
    
    // Private helper methods...
}
```

## 9. Configuration Parameter Guidelines

### Basic I2C Sensors
For simple sensors that only need basic I2C communication:
```rust
struct Config {
    network_port: u16,
    bind_address: String,
    i2c_bus_device_path: String,
    i2c_address_decimal: u16,
}
```

### Advanced Configurable Sensors
For sensors with configurable parameters:
```rust
struct Config {
    network_port: u16,
    bind_address: String,
    i2c_bus_device_path: String,
    i2c_address_decimal: u16,
    // Sensor-specific configuration
    gain: u16,
    integration_time: u16,
    // Additional parameters as needed
}
```

### Non-I2C Sensors
For sensors that don't use I2C addresses:
```rust
struct Config {
    network_port: u16,
    bind_address: String,
    i2c_bus_device_path: String,
    // No i2c_address_decimal field
    // Sensor-specific parameters
}
```

## 10. Deployment Standards

### Build Configuration
- **Target**: ARM Linux (`arm-unknown-linux-musleabihf`)
- **Release Mode**: Always deploy release builds
- **Binary Location**: `target/arm-unknown-linux-musleabihf/release/{sensor_name}_api`

### Deployment Structure
```
/opt/sensor-api/
├── bme280/
│   ├── target/arm-unknown-linux-musleabihf/release/bme280_api
│   └── config.json
├── ltr390/
│   ├── target/arm-unknown-linux-musleabihf/release/ltr390_api
│   └── config.json
└── ...
```

## 11. Testing and Validation

### Compilation Testing
- **Primary**: Test ARM Linux compilation on CI/CD
- **Development**: Windows compilation allowed for syntax validation (expect Linux-specific failures)

### Functional Testing
- Verify each sensor API responds on correct port
- Validate JSON response structure
- Test configuration file generation
- Confirm SystemD service functionality

## 12. Documentation Standards

### Code Comments
- All configuration fields MUST have inline comments
- Complex sensor calculations MUST include explanatory comments
- Error handling MUST include context in log messages

### README Requirements
Each sensor project should include:
- Sensor description and capabilities
- Configuration options
- API endpoint documentation
- Deployment instructions

## 13. Sensor-Collector Integration

### API URL Pattern
- **Format**: `http://localhost:{port}/sensor_data`
- **Registration**: Add to sensor-collector `api_urls` configuration

### Data Flow
1. Sensor APIs expose HTTP endpoints
2. Sensor-collector polls all configured APIs
3. Data aggregated and sent to InfluxDB/MQTT
4. Centralized logging and monitoring

## 14. GitHub Actions CI/CD Standards

### Workflow Overview
The project uses an automated CI/CD pipeline that builds and releases individual sensors when changes are detected or when no previous release exists.

### Workflow Triggers
- **Push Events**: Triggered on pushes to `main` and `dev` branches
- **Pull Requests**: Triggered on PRs targeting `main` branch
- **Release Creation**: Only occurs on `main` branch pushes

### Change Detection Logic
The workflow automatically detects which sensors need building:

1. **File Change Detection**: Compares current commit with previous to identify modified sensor directories
2. **Release Gap Detection**: On `main` branch, checks for sensors without existing releases
3. **Smart Building**: Only builds sensors that have changes OR lack releases

### Sensor Registry
All sensors must be registered in the workflow's sensor list:
```yaml
ALL_SENSORS=("bme280" "pmsa003i" "scd-41" "ltr390" "bh1750" "veml7700" "tsl2591" "sensor-collector")
```

### Binary Naming Convention
The workflow expects specific binary names based on sensor type:
- **Standard Sensors**: `{sensor_name}_api` (e.g., `bme280_api`)
- **Special Cases**: 
  - `sensor-collector` → `sensor-collector`
  - `scd-41` → `scd-41_api`

### Version Format
- **Pattern**: `YYYY.MM.DD-HH.MM` (24-hour format)
- **Generation**: UTC timestamp at build time
- **Example**: `2025.08.19-14.30`

### Release Structure
Each release includes:
```
{sensor_name}-{version}-arm-linux.tar.gz
├── {binary_name}                    # Compiled ARM binary
├── config.json.example              # Sample configuration
├── {sensor_name}.service            # SystemD service file
└── README.md                        # Installation instructions
```

### Required Project Structure for CI/CD
For a sensor to be compatible with the automated workflow:

#### 1. Directory Structure
```
{sensor_name}/
├── Cargo.toml                       # Must define correct binary name
├── config.json                      # Optional: included as example
└── src/
    └── main.rs                      # Entry point
```

#### 2. Cargo.toml Requirements
```toml
[package]
name = "{sensor_name}_api"

[[bin]]
name = "{sensor_name}_api"           # Must match expected naming convention
path = "src/main.rs"

[dependencies]
# Standard dependencies as per guidelines
```

#### 3. SystemD Service File
Must exist in `systemd/{sensor_name}.service` relative to repository root

### Adding New Sensors to CI/CD

To add a new sensor to the automated build pipeline:

1. **Register in Workflow**: Add sensor name to `ALL_SENSORS` array in `.github/workflows/rust.yml`
2. **Follow Naming**: Ensure binary name matches expected convention
3. **Create SystemD Service**: Add service file to `systemd/` directory
4. **Test Build**: Verify sensor builds successfully for ARM target

### Build Targets and Cross-Compilation
- **Primary Target**: `arm-unknown-linux-musleabihf`
- **Dependencies**: Automatically installs cross-compilation tools
- **Libraries**: Configured for ARM Linux embedded systems

### Release Management
- **Automatic Releases**: Created only on `main` branch
- **Tag Format**: `{sensor_name}-{version}` (e.g., `bme280-2025.08.19-14.30`)
- **Latest Tags**: `{sensor_name}-latest` (auto-updated to point to most recent release)
- **Release Notes**: Auto-generated with build information
- **Artifacts**: Compressed archive with all deployment files

#### Tag Types
1. **Version Tags**: `{sensor_name}-{timestamp}` - Immutable, specific release versions
2. **Latest Tags**: `{sensor_name}-latest` - Mutable, always points to most recent release

#### Usage Examples
```bash
# Download specific version
wget https://github.com/owner/repo/releases/download/bme280-2025.08.19-14.30/bme280-2025.08.19-14.30-arm-linux.tar.gz

# Download latest version (always current)
wget https://github.com/owner/repo/releases/download/bme280-latest/bme280-2025.08.19-14.30-arm-linux.tar.gz
```

### Build Matrix Strategy
- **Parallel Building**: Each sensor builds independently
- **Fail-Fast Disabled**: One sensor failure doesn't stop others
- **Conditional Execution**: Only builds sensors with changes or missing releases

### Integration with Development Workflow

#### For New Sensor Development:
1. Create sensor following project standards
2. Add to `ALL_SENSORS` array in workflow
3. Push to `dev` branch for build testing
4. Merge to `main` for automatic release

#### For Sensor Updates:
1. Modify sensor code in feature branch
2. Create PR to trigger validation build
3. Merge to `main` triggers automatic release with new timestamp version

### Troubleshooting CI/CD Issues

#### Common Build Failures:
- **Binary Not Found**: Check Cargo.toml binary name matches convention
- **Missing Dependencies**: Ensure all required crates are in Cargo.toml
- **Cross-Compilation Errors**: Verify ARM-compatible dependencies

#### Release Issues:
- **Missing SystemD Service**: Ensure service file exists in `systemd/` directory
- **Permission Errors**: Verify GitHub token has appropriate permissions
- **Duplicate Releases**: Check if sensor already has release for same timestamp

### Monitoring and Notifications
- **Build Summary**: Generated for each workflow run
- **Step Summary**: Detailed information about build decisions
- **Release Notifications**: GitHub automatically notifies on new releases

## 15. Future Considerations

### Planned Enhancements
- Health check endpoints (`/health`)
- Metrics endpoints (`/metrics`)
- Configuration reload without restart
- Sensor calibration endpoints

### Scalability
- Port range allows for 50 sensor APIs
- Configuration supports multiple instances per sensor type
- SystemD services enable independent lifecycle management

---

## Compliance Checklist

When implementing a new sensor API, verify:

- [ ] Port assigned from available range (5000-5049)
- [ ] Standard configuration structure implemented
- [ ] Cargo.toml follows naming conventions
- [ ] SystemD service file created
- [ ] API returns standardized JSON response
- [ ] Error handling uses appropriate HTTP status codes
- [ ] Logging implemented with proper levels
- [ ] Code organization follows standards
- [ ] Documentation includes configuration options
- [ ] Integration tested with sensor-collector
- [ ] **GitHub Actions**: Sensor added to workflow `ALL_SENSORS` array
- [ ] **GitHub Actions**: Binary name follows expected convention
- [ ] **GitHub Actions**: SystemD service file exists in correct location
- [ ] **GitHub Actions**: Builds successfully for ARM Linux target
- [ ] **GitHub Actions**: Release artifacts include all required files

## Contact and Maintenance

This document should be updated whenever:
- New sensor APIs are added
- Port assignments change
- Configuration standards evolve
- Deployment procedures are modified

Maintain this document alongside the codebase to ensure ongoing consistency across all sensor implementations.
