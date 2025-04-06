<p align="center">
<img width="128" src="https://github.com/omnius-labs/opxs-apps-rs/blob/main/docs/logo.png?raw=true" alt="Opxs logo">
</p>

<h1 align="center">Opxs - Omnius Anything Service (Apps)</h1>

[![test](https://github.com/omnius-labs/opxs-apps-rs/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/omnius-labs/opxs-apps-rs/actions/workflows/test.yml)

This repository contains the backend code for the Opxs platform, a suite of experimental web services written in Rust.

- Frontend: https://github.com/omnius-labs/opxs-web-ts

## Features

- **Image Converter**: Convert images between different formats online.

## Development

### Getting Started

To run the Opxs API locally, you need to set up your environment first.

### Requirements

- Cargo (Rust's package manager)
- Docker

### Running Locally

#### 1. Start PostgreSQL:

```sh
docker compose up --build
```

#### 2. Set the image converter directory path:

Download the zip file from the following URL and place it in a local directory.

https://github.com/omnius-labs/image-converter-cs/releases

Set the path of the placed directory as an environment variable.

```sh
export IMAGE_CONVERTER_DIR="xxx"
export GOOGLE_AUTH_CLIENT_ID="xxx"
export GOOGLE_AUTH_CLIENT_SECRET="xxx"
```

#### 3. Start the Opxs API:

```sh
RUN_MODE=local cargo make watch
```

Access the API documentation locally at:
https://localhost.omnius-labs.com/api/docs/

## Links

- Official Documentation: https://docs.omnius-labs.com/

## License

This project is released under the MIT License. For more details, please refer to the [LICENSE](LICENSE.txt) file.

## Contribution

If you would like to contribute to this project, please contact us through [Issues](https://github.com/omnius-labs/axus-daemon-rs/issues) or [Pull Requests](https://github.com/omnius-labs/axus-daemon-rs/pulls) on GitHub.
