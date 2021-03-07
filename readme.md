# Blockade Recon
A sequel to [Blockade Recon](https://github.com/cassa/blockade-recon). A tool for capturing and analysing wireless network traffic.

![An example of blockade recon](example.png)

# Goals
- Tally devices by manufacturer
- Collect as much information as possible
- Save sessions
- Analyse and map probable device connections
- Basic demonstartions of 802.11 attacks
- Attempt mapping devices in space using multiple devices

# Installing

```sh
$ cargo install --git https://github.com/AidoP/blockade-recon
```

## Dependencies
- libpcap
- Cargo and Rust

# Usage

```sh
$ blockade-recon --help
$ blockade-recon -i
```