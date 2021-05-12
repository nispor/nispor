# Copyright 2020-2021 Red Hat
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


class NisporEthtool:
    def __init__(self, info):
        if "pause" in info:
            self._pause = NisporEthtoolPause(info["pause"])
        else:
            self._pause = None

        if "features" in info:
            self._features = NisporEthtoolFeatures(info["features"])
        else:
            self._features = None

    @property
    def pause(self):
        return self._pause

    @property
    def features(self):
        return self._features

class NisporEthtoolPause:
    def __init__(self, info):
        self._info = info

    @property
    def rx(self):
        return self._info["rx"]

    @property
    def tx(self):
        return self._info["tx"]

    @property
    def auto_negotiate(self):
        return self._info["auto_negotiate"]

class NisporEthtoolFeatures:
    def __init__(self, info):
        self._info = info

    @property
    def fixed(self):
        return self._info["fixed"]

    @property
    def changeable(self):
        return self._info["changeable"]
