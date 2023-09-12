# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporMacsec(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._macsec_info = self._info.get("macsec", {})

    @property
    def sci(self):
        return self._macsec_info["sci"]

    @property
    def port(self):
        return self._macsec_info["port"]

    @property
    def icv_len(self):
        return self._macsec_info["icv_len"]

    @property
    def cipher(self):
        return self._macsec_info["cipher"]

    @property
    def window(self):
        return self._macsec_info["window"]

    @property
    def encoding_sa(self):
        return self._macsec_info["encoding_sa"]

    @property
    def encrypt(self):
        return self._macsec_info["encrypt"]

    @property
    def protect(self):
        return self._macsec_info["protect"]

    @property
    def send_sci(self):
        return self._macsec_info["send_sci"]

    @property
    def end_station(self):
        return self._macsec_info["end_station"]

    @property
    def scb(self):
        return self._macsec_info["scb"]

    @property
    def replay_protect(self):
        return self._macsec_info["replay_protect"]

    @property
    def validate(self):
        return self._macsec_info["validate"]

    @property
    def offload(self):
        return self._macsec_info["offload"]

    @property
    def base_iface(self):
        return self._macsec_info["base_iface"]
