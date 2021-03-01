The nispor(Network Inspector) project is designed to providing unified
interface for Linux network state querying.

Currently providing:
 * Rust crate
 * Python binding
 * C binding
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

### Print route entries

```bash
npc route
```

## Supported features
 * IPv4/IPv6 address
 * Bond
 * Linux Bridge
 * Linux Bridge VLAN filtering
 * VLAN
 * VxLAN
 * Route
 * Dummy
 * TUN/TAP
 * Veth
 * VRF(Virtual Routing and Forwarding)
 * SR-IOV
 * MacVlan
 * MacVtap

## TODO:
 * Error handling instead of `unwrap()/panic!/etc`
 * SR-IOV VF-PF relation is possible
 * VLAN QoS
 * Route rule
 * Traffic control
 * Manpage for npc/npd
 * pkgconfig file for nispor C library

## What should nispor not do
To make nispor only to small good things, this is the list of things
could be done by nispor but should not do:
 * Ordering the network interface configuration base on child/parent,
   controller/port relationships.
 * Wrapping of multiple kernel options into simple ones.
 * User space networking.
 * Notification on network change.
