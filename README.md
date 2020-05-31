## Design

### Rust module

Path: `src/lib`

### Command line tool

Path: `src/cli`

Take first argument as NIC name to print network status, if no no argument
defined, print out full network status.

### Varlink service

Path: `src/varlink`

## Usage

### Print all network status

```bash
make debug
```

### Print network status of certain NIC

```bash
ARGS="bond99" make debug
```

### Varlink service

```bash
make srv
```

### Varlink client

```bash
# Please install `libvarlink-util` pacakge beforehand
make cli
```

## Goal

 * Provide C/Python/Rust binding to query linux network status
 * Provide varlink interface for querying linux networks status

## Supported interface types
 * Bond
 * Linux Bridge
 * VLAN

## TODO:
 * Error handling instead of `unwrap()/panic!/etc`
 * VLAN QoS
 * VxLAN
 * Dummy
 * Veth
 * SR-IOV
 * Route
 * Route rule
 * OpenvSwitch
 * Plugin design
