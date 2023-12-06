# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporHsr(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._hsr_info = self._info.get("hsr", {})

    @property
    def port1(self):
        return self._hsr_info["port1"]

    @property
    def port2(self):
        return self._hsr_info["port2"]

    @property
    def supervision_addr(self):
        return self._hsr_info["supervision_addr"]

    @property
    def seq_nr(self):
        return self._hsr_info["seq_nr"]

    @property
    def multicast_spec(self):
        return self._hsr_info["multicast_spec"]

    @property
    def version(self):
        return self._hsr_info["version"]

    @property
    def protocol(self):
        return self._hsr_info["protocol"]
