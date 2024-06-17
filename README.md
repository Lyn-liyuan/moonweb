**Moonweb: LLM Chat Tool**

Welcome to Moonweb, a web chat tool developed with Rust, Dioxus, and Candle frameworks that supports a variety of open-source Large Language Models (LLMs). This project aims to provide a dynamic and flexible platform for integrating and testing different LLMs.

**Features**

- **Multi-Model Support**: Seamless integration of various open-source LLMs.
- **Dynamic Model Loading**: Supports dynamic loading and unloading of models at runtime.
- **Independent Process Isolation**: Each model runs in an independent process, providing services through ZeroMQ, ensuring stability and responsiveness.
- **Web Interface**: A responsive and user-friendly web interface built with the Dioxus framework.
- **Open Source**: Fully open source, encouraging community contributions and customization.

**Quick Start**

1. **Install Rust**: Ensure that Rust is installed on your system. Visit the [Rust official website](https://www.rust-lang.org/) for installation instructions.
1. **Install Dioxus**: dioxus is a react and vue like web framework. Visit the [document ](https://dioxuslabs.com/learn/0.5/getting_started)of dioxus.
1. **Clone the Repository**: Clone the Moonweb project to your local machine using Git.

   git clone https://github.com/ Lyn-liyuan/moonweb.git

1. **Build the Project**: Navigate to the project directory and build the project using Cargo.
1. cd moonweb
```shell
   cargo build
```
1. **Run the Services**: Start the LLM model services.
```shell
   cargo run –-release -- --server master
```
1. **Run the Web** : Start the web services.
```shell
   dx serve --platform fullstack
```
**Architecture Overview**

- **Frontend**: The web interface built with Dioxus, responsible for displaying chat content and user input.
- **Backend**: Rust backend services that handle web requests and communicate with LLM model services.
- **Model Services**: Each LLM model runs as an independent process, communicating with the backend service via ZeroMQ.

**Model Integration**

To integrate a new LLM model, follow these steps:

1. Create a model service process that implements ZeroMQ communication.
1. Register the communication interface of the model service in the backend service.
1. Update the frontend interface to support input and output for the new model.

**Contributing**

We welcome contributions in any form, including but not limited to:

- Code submissions
- Feature requests
- Bug reports
- Documentation improvements



**Known Limitations**

- Currently, Moonweb does not support Server Sent Events (SSE), which means that chat responses will be displayed all at once on the web after the model generates them.

**License**

This project is licensed under the "MIT License".

**Contact**

- Project Maintainer: [LYN]
- Email: [yuanli13@asu.edu]
- GitHub: [@yourusername](https://github.com/Lyn-liyuan)
