# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface
from .bond import NisporBond
from .bond import NisporBondSubordinate
from .bridge import NisporBridge
from .bridge import NisporBridgePort
from .hsr import NisporHsr
from .ipoib import NisporIpoib
from .mac_vlan import NisporMacVlan
from .mac_vtap import NisporMacVtap
from .macsec import NisporMacsec
from .tun import NisporTun
from .veth import NisporVeth
from .vlan import NisporVlan
from .vrf import NisporVRF
from .vxlan import NisporVxlan


class NisporIfaceState:
    def __init__(self, info):
        self._ifaces = {}
        if info:
            for iface_name, iface_info in info.items():
                self._ifaces[iface_name] = _iface_info_to_obj(iface_info)

    def __str__(self):
        return f"{self._ifaces}"

    def get(self, iface_name):
        return self._ifaces.get(iface_name)

    def __getitem__(self, iface_name):
        return self._ifaces[iface_name]

    def keys(self):
        for iface in self._ifaces.keys():
            yield iface

    def values(self):
        for iface in self._ifaces.values():
            yield iface


def _iface_info_to_obj(iface_info):
    iface_type = iface_info["iface_type"]
    ctrl_type = iface_info.get("controller_type")
    if iface_type == "bond":
        iface = NisporBond(iface_info)
    elif iface_type == "bridge":
        iface = NisporBridge(iface_info)
    elif iface_type == "tun":
        iface = NisporTun(iface_info)
    elif iface_type == "vlan":
        iface = NisporVlan(iface_info)
    elif iface_type == "vxlan":
        iface = NisporVxlan(iface_info)
    elif iface_type == "veth":
        iface = NisporVeth(iface_info)
    elif iface_type == "vrf":
        iface = NisporVRF(iface_info)
    elif iface_type == "mac_vlan":
        iface = NisporMacVlan(iface_info)
    elif iface_type == "mac_vtap":
        iface = NisporMacVtap(iface_info)
    elif iface_type == "Ipoib":
        iface = NisporIpoib(iface_info)
    elif iface_type == "macsec":
        iface = NisporMacsec(iface_info)
    elif iface_type == "hsr":
        iface = NisporHsr(iface_info)
    else:
        iface = NisporBaseIface(iface_info)
    if ctrl_type == "bond":
        iface.subordinate_state = NisporBondSubordinate(iface_info)
    elif ctrl_type == "bridge":
        iface.subordinate_state = NisporBridgePort(iface_info)
    return iface
