The nispor(Network Inspector) project is designed to providing unified
interface for Linux network state querying.

Currently providing:
 * Rust crate
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
npc iface bond99
```

### Print route entries

```bash
npc route
```
## What should nispor not do
To make nispor only to small good things, this is the list of things
could be done by nispor but should not do:
 * Ordering the network interface configuration base on child/parent,
   controller/port relationships.
 * Wrapping of multiple kernel options into simple ones.
 * User space networking.
 * Notification on network change.
