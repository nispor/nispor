# SPDX-License-Identifier: Apache-2.0


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

        if "link_mode" in info:
            self._link_mode = NisporEthtoolLinkMode(info["link_mode"])
        else:
            self._link_mode = None

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

    @property
    def link_mode(self):
        return self._link_mode


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


class NisporEthtoolLinkMode:
    def __init__(self, info):
        self._info = info

    @property
    def auto_negotiate(self):
        return self._info["auto_negotiate"]

    @property
    def ours(self):
        return self._info["ours"]

    @property
    def peer(self):
        return self._info.get("peer")

    @property
    def speed(self):
        return self._info["speed"]

    @property
    def duplex(self):
        return self._info["duplex"]

    @property
    def controller_subordinate_cfg(self):
        return self._info.get("controller_subordinate_cfg")

    @property
    def controller_subordinate_state(self):
        return self._info.get("controller_subordinate_state")

    @property
    def lanes(self):
        return self._info.get("lanes")
