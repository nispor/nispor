# SPDX-License-Identifier: Apache-2.0


class NisporMptcpState:
    def __init__(self, info):
        self._addrs = [
            NisporMptcpAddress(addr_info) for addr_info in info.get("addresses", [])
        ]

    @property
    def enabled(self):
        return self._info["enabled"]

    @property
    def add_addr_accepted_limit(self):
        return self._info.get("add_addr_accepted_limit")

    @property
    def subflows_limit(self):
        return self._info.get("subflows_limit")

    @property
    def addresses(self):
        return self._addrs


class NisporMptcpAddress:
    def __init__(self, info):
        self._info = info

    @property
    def address(self):
        return self._info["address"]

    @property
    def id(self):
        return self._info.get("id")

    @property
    def port(self):
        return self._info.get("port")

    @property
    def flags(self):
        return self._info.get("flags")

    @property
    def iface(self):
        return self._info.get("iface")
