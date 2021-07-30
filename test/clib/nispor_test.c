#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <nispor.h>

int apply(const char* state) {
    char *err_kind = NULL;
	char *err_msg = NULL;
    printf("state: %s", state);
    int rc = nispor_net_state_apply(state, &err_kind, &err_msg);
    if (rc != EXIT_SUCCESS) {
        fprintf(stderr, "err_msg: %s\n", err_msg);
        fprintf(stderr, "err_kind: %s\n", err_kind); 
        fprintf(stderr, "state: %s\n", state);
    }
	nispor_err_kind_free(err_kind);
	nispor_err_msg_free(err_msg);
    return  rc;
}

bool test_retrieve() {
    char *err_kind = NULL;
	char *err_msg = NULL;
	char *state = NULL;
	int rc = nispor_net_state_retrieve(&state, &err_kind, &err_msg);
    if (rc != EXIT_SUCCESS) {
        fprintf(stderr, "err_msg: %s\n", err_msg);
        fprintf(stderr, "err_kind: %s\n", err_kind); 
    }
	nispor_net_state_free(state);
	nispor_err_kind_free(err_kind);
	nispor_err_msg_free(err_msg);
    return rc == EXIT_SUCCESS;
}

bool test_apply() {
    const char* create_veth =
"{\n"
"  \"ifaces\": [\n"
"    {\n"
"      \"name\": \"veth1\",\n"
"      \"type\": \"veth\",\n"
"      \"veth\": {\n"
"          \"peer\": \"veth1.ep\"\n"
"      }\n"
"    },\n"
"    {\n"
"      \"name\": \"veth1.ep\",\n"
"      \"type\": \"veth\"\n"
"    }\n"
"  ]\n"
"}";

    const char* remove_veth =
"{\n"
"  \"ifaces\": [\n"
"    {\n"
"      \"name\": \"veth1\",\n"
"      \"type\": \"veth\",\n"
"      \"state\": \"absent\"\n"
"    }\n"
"  ]\n"
"}";
    int rc_create = apply(create_veth);

    // Tear down veth
    int rc_remove = apply(remove_veth);

    return rc_create == EXIT_SUCCESS && rc_remove == EXIT_SUCCESS;
}

bool test_apply_bad_yaml() {
    return apply("{") != EXIT_SUCCESS;
}

       

int main() {
    
    struct test {                                                                   
        const char* description;
        bool (*runner)(void);
    }; 

    struct test test_list[] = {
       { "Calling nispor_net_state_retrieve should success", test_retrieve },
       { "Calling nispor_net_state_apply should success", test_apply },
       { "Calling nispor_net_state_apply with bad JSON should fail", test_apply_bad_yaml },
    };

    for (long unsigned int i = 0; i < sizeof(test_list) / sizeof(struct test); i++) {
        if (!(*test_list[i].runner)()){
            fprintf(stderr, "[FAILURE]: %s\n", test_list[i].description);
            return EXIT_FAILURE;
        } else {
            fprintf(stderr, "[SUCCESS]: %s\n", test_list[i].description);
        }
    };
}
