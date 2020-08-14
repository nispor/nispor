/*
 * Copyright 2020 Red Hat
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


#ifndef _LIBNISPOR_H_
#define _LIBNISPOR_H_

#ifdef __cplusplus
extern "C" {
#endif

#define NISPOR_VERSION              "0.2.1"
#define NISPOR_VERSION_MAJOR        0
#define NISPOR_VERSION_MINOR        2
#define NISPOR_VERSION_MICRO        1

#define NISPOR_PASS                 0
#define NISPOR_FAIL                 1

/**
 * nispor_state_get - Get network state
 *
 * Version:
 *      0.2
 *
 * Description:
 *      Get network state in the format of JSON.
 *
 * @state:
 *      Output pointer of char array for network state in json format.
 *      The memory should be freed by nispor_state_free().
 * @err_kind:
 *      Output pointer of char array for error kind.
 *      The memory should be freed by nispor_err_kind_free().
 * @err_msg:
 *      Output pointer of char array for error message.
 *      The memory should be freed by nispor_err_msg_free().
 *
 * Return:
 *      Error code:
 *          * NISPOR_PASS
 *              On success.
 *          * NISPOR_FAIL
 *              On failure.
 */
int nispor_state_get(char **state, char **err_kind, char **err_msg);

void nispor_state_free(char *state);

void nispor_err_msg_free(char *err_msg);

void nispor_err_kind_free(char *err_kind);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* End of _LIBNISPOR_H_ */
