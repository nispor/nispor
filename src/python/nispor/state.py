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

import json

from .clib_wrapper import retrieve_net_state_json
from .dns import NisporDnsState
from .iface import NisporIfaceState
from .route import NisporRouteState
from .route_rule import NisporRouteRuleState


class NisporNetState:
    def __str__(self):
        return f"{self._info}"

    def __init__(self, info):
        self._info = info
        self._ifaces = NisporIfaceState(info.get("ifaces"))
        self._routes = NisporRouteState(info.get("routes"))
        self._route_rules = NisporRouteRuleState(info.get("rules"))
        self._dns = NisporDnsState(info.get("dns_resolver"))

    @property
    def ifaces(self):
        return self._ifaces

    @property
    def routes(self):
        return self._routes

    @property
    def route_rules(self):
        return self._route_rules

    @property
    def dns_resolver(self):
        return self._dns

    @staticmethod
    def retrieve():
        return NisporNetState(json.loads(retrieve_net_state_json()))
