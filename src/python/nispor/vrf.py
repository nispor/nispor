# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface
from .base_iface import NisporBaseSubordinateIface


class NisporVRF(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._vrf_info = self._info.get("vrf", {})

    @property
    def subordinates(self):
        return self._vrf_info.get("subordinates", [])

    @property
    def table_id(self):
        return self._vrf_info.get("table_id")
