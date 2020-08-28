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

import ctypes
from ctypes import c_int, c_char_p, c_uint32, Structure, POINTER, byref
from ctypes.util import find_library

lib = ctypes.cdll.LoadLibrary(find_library("nispor"))

lib.nispor_net_state_retrieve.restype = c_int
lib.nispor_net_state_retrieve.argtypes = (
    POINTER(c_char_p),
    POINTER(c_char_p),
    POINTER(c_char_p),
)
lib.nispor_net_state_free.restype = None
lib.nispor_net_state_free.argtypes = (c_char_p,)
lib.nispor_err_kind_free.restype = None
lib.nispor_err_kind_free.argtypes = (c_char_p,)
lib.nispor_err_msg_free.restype = None
lib.nispor_err_msg_free.argtypes = (c_char_p,)


class NisporError(Exception):
    def __init__(self, kind, msg):
        self.kind = kind
        self.msg = msg
        super().__init__(f"{kind}: {msg}")


def retrieve_net_state_json():
    c_err_msg = c_char_p()
    c_err_kind = c_char_p()
    c_state = c_char_p()
    rc = lib.nispor_net_state_retrieve(
        byref(c_state), byref(c_err_kind), byref(c_err_msg)
    )
    state = c_state.value
    err_msg = c_err_msg.value
    err_kind = c_err_kind.value
    lib.nispor_net_state_free(c_state)
    lib.nispor_err_kind_free(c_err_kind)
    lib.nispor_err_msg_free(c_err_msg)
    if rc != 0:
        raise NisporError(err_kind, err_msg)
    return state.decode("utf-8")
