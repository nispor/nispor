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


class NisporRouteState:
    def __init__(self, info):
        self._info = info

    @property
    def address_family(self):
        return self._info.get("address_family")

    @property
    def tos(self):
        return self._info.get("tos")

    @property
    def table(self):
        return self._info.get("table")

    @property
    def protocol(self):
        return self._info.get("protocol")

    @property
    def scope(self):
        return self._info.get("scope")

    @property
    def route_type(self):
        return self._info.get("route_type")

    @property
    def flags(self):
        return self._info.get("flags")

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
