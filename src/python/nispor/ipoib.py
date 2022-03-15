# Copyright 2021-2022 Red Hat
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

class NisporIpoib(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._ipoib_info = self._info.get("ipoib", {})

    @property
    def pkey(self):
        return self._ipoib_info["pkey"]

    @property
    def mode(self):
        return self._ipoib_info["mode"]

    @property
    def umcast(self):
        return self._ipoib_info["umcast"]
