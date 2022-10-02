# SPDX-License-Identifier: Apache-2.0


class NisporRouteState:
    def __init__(self, info):
        self._rts = [NisporRoute(rt_info) for rt_info in info]

    def __iter__(self):
        for rt in self._rts:
            yield rt


class NisporRoute:
    def __init__(self, info):
        self._info = info

    @property
    def address_family(self):
        return self._info["address_family"]

    @property
    def tos(self):
        return self._info["tos"]

    @property
    def table(self):
        return self._info["table"]

    @property
    def protocol(self):
        return self._info["protocol"]

    @property
    def scope(self):
        return self._info["scope"]

    @property
    def route_type(self):
        return self._info["route_type"]

    @property
    def flags(self):
        return self._info["flags"]

    @property
    def dst(self):
        return self._info.get("dst")

    @property
    def oif(self):
        return self._info.get("oif")

    @property
    def iif(self):
        return self._info.get("iif")

    @property
    def prefered_src(self):
        return self._info.get("prefered_src")

    @property
    def src(self):
        return self._info.get("src")

    @property
    def class_id(self):
        return self._info.get("class_id")

    @property
    def gateway(self):
        return self._info.get("gateway")

    @property
    def via(self):
        return self._info.get("via")

    @property
    def mark(self):
        return self._info.get("mark")

    @property
    def uid(self):
        return self._info.get("uid")

    @property
    def lock(self):
        return self._info.get("lock")

    @property
    def mtu(self):
        return self._info.get("mtu")

    @property
    def window(self):
        return self._info.get("window")

    @property
    def rtt(self):
        return self._info.get("rtt")

    @property
    def rttvar(self):
        return self._info.get("rttvar")

    @property
    def ssthresh(self):
        return self._info.get("ssthresh")

    @property
    def cwnd(self):
        return self._info.get("cwnd")

    @property
    def advmss(self):
        return self._info.get("advmss")

    @property
    def reordering(self):
        return self._info.get("reordering")

    @property
    def hoplimit(self):
        return self._info.get("hoplimit")

    @property
    def initcwnd(self):
        return self._info.get("initcwnd")

    @property
    def features(self):
        return self._info.get("features")

    @property
    def rto_min(self):
        return self._info.get("rto_min")

    @property
    def initrwnd(self):
        return self._info.get("initrwnd")

    @property
    def quickack(self):
        return self._info.get("quickack")

    @property
    def cc_algo(self):
        return self._info.get("cc_algo")

    @property
    def fastopen_no_cookie(self):
        return self._info.get("fastopen_no_cookie")

    @property
    def cache_clntref(self):
        return self._info.get("cache_clntref")

    @property
    def cache_last_use(self):
        return self._info.get("cache_last_use")

    @property
    def cache_expires(self):
        return self._info.get("cache_expires")

    @property
    def cache_error(self):
        return self._info.get("cache_error")

    @property
    def cache_used(self):
        return self._info.get("cache_used")

    @property
    def cache_id(self):
        return self._info.get("cache_id")

    @property
    def cache_ts(self):
        return self._info.get("cache_ts")

    @property
    def cache_ts_age(self):
        return self._info.get("cache_ts_age")

    @property
    def metric(self):
        return self._info.get("metric")

    @property
    def perf(self):
        return self._info.get("perf")

    @property
    def multipath(self):
        mp_rts = self._info.get("multipath")
        if mp_rts:
            return [NisporMultipathRoute(m) for m in mp_rts]
        else:
            None


class NisporMultipathRoute:
    def __init__(self, info):
        self._info = info

    @property
    def via(self):
        return self._info["via"]

    @property
    def iface(self):
        return self._info["iface"]

    @property
    def weight(self):
        return self._info["weight"]

    @property
    def flags(self):
        return self._info["flags"]
