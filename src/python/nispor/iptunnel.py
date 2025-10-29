# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporIpTunnel(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._ip_tunnel_info = self._info.get("iptunnel", {})

    @property
    def mode(self):
        return self._ip_tunnel_info.get("mode")

    @property
    def parent(self):
        return self._ip_tunnel_info.get("parent")

    @property
    def local(self):
        return self._ip_tunnel_info.get("local")

    @property
    def remote(self):
        return self._ip_tunnel_info.get("remote")

    @property
    def ttl(self):
        return self._ip_tunnel_info.get("ttl")

    @property
    def tos(self):
        return self._ip_tunnel_info.get("tos")

    @property
    def ip6tun_flags(self):
        return self._ip_tunnel_info.get("ip6tun_flags")

    @property
    def pmtu_disc(self):
        return self._ip_tunnel_info.get("pmtu_disc")

    @property
    def encap_limit(self):
        return self._ip_tunnel_info.get("encap_limit")

    @property
    def flow_info(self):
        return self._ip_tunnel_info.get("flow_info")

    @property
    def encap_type(self):
        return self._ip_tunnel_info.get("encap_type")

    @property
    def encap_flags(self):
        return self._ip_tunnel_info.get("encap_flags")

    @property
    def encap_source_port(self):
        return self._ip_tunnel_info.get("encap_source_port")

    @property
    def encap_destination_port(self):
        return self._ip_tunnel_info.get("encap_destination_port")

    @property
    def collect_metadata(self):
        return self._ip_tunnel_info.get("collect_metadata")

    @property
    def fwmark(self):
        return self._ip_tunnel_info.get("fwmark")
