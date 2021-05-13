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

        if "coalesce" in info:
            self._coalesce = NisporEthtoolCoalesce(info["coalesce"])
        else:
            self._coalesce = None

        if "ring" in info:
            self._ring = NisporEthtoolRing(info["ring"])
        else:
            self._ring = None

    @property
    def pause(self):
        return self._pause

    @property
    def features(self):
        return self._features

    @property
    def coalesce(self):
        return self._coalesce

    @property
    def ring(self):
        return self._ring


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


class NisporEthtoolCoalesce:
    def __init__(self, info):
        self._info = info

    @property
    def pkt_rate_high(self):
        return self._info.get("pkt_rate_high")

    @property
    def pkt_rate_low(self):
        return self._info.get("pkt_rate_low")

    @property
    def rate_sample_interval(self):
        return self._info.get("rate_sample_interval")

    @property
    def rx_max_frames(self):
        return self._info.get("rx_max_frames")

    @property
    def rx_max_frames_high(self):
        return self._info.get("rx_max_frames_high")

    @property
    def rx_max_frames_irq(self):
        return self._info.get("rx_max_frames_irq")

    @property
    def rx_max_frames_low(self):
        return self._info.get("rx_max_frames_low")

    @property
    def rx_usecs(self):
        return self._info.get("rx_usecs")

    @property
    def rx_usecs_high(self):
        return self._info.get("rx_usecs_high")

    @property
    def rx_usecs_irq(self):
        return self._info.get("rx_usecs_irq")

    @property
    def rx_usecs_low(self):
        return self._info.get("rx_usecs_low")

    @property
    def stats_block_usecs(self):
        return self._info.get("stats_block_usecs")

    @property
    def tx_max_frames(self):
        return self._info.get("tx_max_frames")

    @property
    def tx_max_frames_high(self):
        return self._info.get("tx_max_frames_high")

    @property
    def tx_max_frames_irq(self):
        return self._info.get("tx_max_frames_irq")

    @property
    def tx_max_frames_low(self):
        return self._info.get("tx_max_frames_low")

    @property
    def tx_usecs(self):
        return self._info.get("tx_usecs")

    @property
    def tx_usecs_high(self):
        return self._info.get("tx_usecs_high")

    @property
    def tx_usecs_irq(self):
        return self._info.get("tx_usecs_irq")

    @property
    def tx_usecs_low(self):
        return self._info.get("tx_usecs_low")

    @property
    def use_adaptive_rx(self):
        return self._info.get("use_adaptive_rx")

    @property
    def use_adaptive_tx(self):
        return self._info.get("use_adaptive_tx")


class NisporEthtoolRing:
    def __init__(self, info):
        self._info = info

    @property
    def rx(self):
        return self._info.get("rx")

    @property
    def rx_max(self):
        return self._info.get("rx_max")

    @property
    def rx_jumbo(self):
        return self._info.get("rx_jumbo")

    @property
    def rx_jumbo_max(self):
        return self._info.get("rx_jumbo_max")

    @property
    def rx_mini(self):
        return self._info.get("rx_mini")

    @property
    def rx_mini_max(self):
        return self._info.get("rx_mini_max")

    @property
    def tx(self):
        return self._info.get("tx")

    @property
    def tx_max(self):
        return self._info.get("tx_max")
