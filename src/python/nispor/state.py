# SPDX-License-Identifier: Apache-2.0

import json

from .clib_wrapper import retrieve_net_state_json
from .iface import NisporIfaceState
from .mptcp import NisporMptcpState
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
        if info.get("mptcp"):
            self._mptcp = NisporMptcpState(info["mptcp"])
        else:
            self._mptcp = None

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
    def mptcp(self):
        return self._mptcp

    @staticmethod
    def retrieve():
        return NisporNetState(json.loads(retrieve_net_state_json()))
