The nispor(Network Inspector) project is designed to providing unified
interface for Linux network state querying.

Currently providing:
 * Rust crate
 * Python binding
 * Varlink interface -- `npd`
 * Command line tool -- `npc`

## Install

```bash
make
sudo PREFIX=/usr make install
```

## Usage

### Print all network status

```bash
npc
```

### Print network status of certain NIC

```bash
npc bond99
```

### Varlink service

```bash
systemctl start nispor.socket
```

### Varlink client

```bash
# Please install `libvarlink-util` pacakge beforehand
varlink call unix:/run/nispor/nispor.so/info.nispor.Get
```

## Supported features
 * IPv4/IPv6 address
 * Bond
 * Linux Bridge
 * Linux Bridge VLAN filtering
 * VLAN
 * VxLAN

## TODO:
 * Error handling instead of `unwrap()/panic!/etc`
 * VLAN QoS
 * Dummy
 * Veth
 * SR-IOV
 * Route
 * Route rule
 * TUN/TAP
 * Traffic control
 * Manpage for npc/npd
