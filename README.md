# Key-Value Service in Rust

This project consists of two services: `kv-service-backend` and `kv-service-frontend`. The backend service communicates with the frontend service using gRPC and is responsible for storing and retrieving key-value pairs. The frontend service exposes the business logic via an HTTP REST API.

## Setup

### Prerequisites

- Rust toolchain installed (https://www.rust-lang.org/tools/install)
- Git
- OpenSSL (for generating TLS certificates)
- Protocol Buffers compiler, along with Protocol Buffers resource files

### Ubuntu

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow the prompts to complete the installation

# Install Git
sudo apt update
sudo apt install git

# Install OpenSSL
sudo apt install openssl

# Install Protocol Buffers compiler and resource files
sudo apt install protobuf-compiler libprotobuf-dev
```

### Fedora

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow the prompts to complete the installation

# Install Git
sudo dnf install git

# Install OpenSSL
sudo dnf install openssl

# Install Protocol Buffers compiler and resource files
sudo dnf install protobuf-compiler protobuf-devel
```


### MacOS

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow the prompts to complete the installation

# Install Git
brew install git

# Install OpenSSL
brew install openssl

# Install Protocol Buffers compiler and resource files
brew install protobuf
```


### Clone the Repository

```bash
git clone https://github.com/weclaw1/kv-service.git
cd kv-service
```

### Generate TLS Certificates (for HTTPS support)

You need to generate TLS certificates for HTTPS support. You can use OpenSSL or any other tool for this purpose.

```bash
# Create a root CA
openssl req -x509 -noenc -subj '/CN=example.com' -newkey rsa:4096 -keyout root.key -out root.crt

# Create a client certificate signing request
openssl req -noenc -newkey rsa:4096 -keyout client.key -out client.csr -subj '/CN=example.com' -addext subjectAltName=DNS:example.com

# Create a server certificate signing request
openssl req -noenc -newkey rsa:4096 -keyout server.key -out server.csr -subj '/CN=example.com' -addext subjectAltName=DNS:example.com

# Sign client CSR using root CA
openssl x509 -req -in client.csr -CA root.crt -CAkey root.key -days 365 -out client.crt -copy_extensions copy

# Sign server CSR using root CA
openssl x509 -req -in server.csr -CA root.crt -CAkey root.key -days 365 -out server.crt -copy_extensions copy
```

Place `server.crt`, `server.key`, `client.crt`, `client.key` and `root.crt` files in the `tls` directory.

### Building the Services
In the `kv-service` folder run

```bash
cargo build
```

## Running the Services

### Backend Service

```bash
# Without TLS
cargo run -p kv-service-backend

# With TLS
TLS=true cargo run -p kv-service-backend
```

### Frontend Service

```bash
# Without TLS
cargo run -p kv-service-frontend

# With TLS
TLS=true cargo run -p kv-service-frontend
```

By default, the backend service runs on `localhost:8081`, and the frontend service runs on `localhost:8080`.
If you want to change the address please change it in .env file or by changing environment variables.

## Usage

### Frontend REST API

The frontend service provides the following REST API endpoints:

- `GET /api/{key}`: Retrieve the value associated with the specified key.
- `PUT /api/{key}`: Update the value associated with the specified key. Request body should be a JSON value, for example `"test"`.
- `DELETE /api/{key}`: Delete the key-value pair associated with the specified key.

### gRPC Communication (Backend Service)

The backend service communicates with the frontend service via gRPC. You can refer to the gRPC protobuf file for message definitions and service methods.

## Testing

### Unit and Integration Tests

Both services have unit and integration tests covering all functionalities. Run tests using:

```bash
cargo test
```

## Contributing

Feel free to contribute to this project by opening issues or pull requests. Your feedback and contributions are highly appreciated.

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/weclaw1/kv-service/blob/main/LICENSE) file for details.