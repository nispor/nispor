#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <nispor.h>

int main(void) {
	int rc = EXIT_SUCCESS;
	char *state = NULL;
	char *err_kind = NULL;
	char *err_msg = NULL;
	if (nispor_net_state_retrieve(&state, &err_kind, &err_msg) ==
	    NISPOR_PASS) {
		printf("%s\n", state);
	} else {
		printf("%s: %s\n", err_kind, err_msg);
		rc = EXIT_FAILURE;
	}

	nispor_net_state_free(state);
	nispor_err_kind_free(err_kind);
	nispor_err_msg_free(err_msg);
	exit(rc);
}
