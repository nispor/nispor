# Copyright 2020 Red Hat
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
