package nispor

// #cgo CFLAGS: -g -Wall
// #cgo LDFLAGS: -L/usr/local/lib64 -lnispor  -Wl,-rpath=/usr/local/lib64
// #include <nispor.h>
// #include <stdlib.h>
import "C"
import "fmt"

func RetrieveNetStateJSON() (string, error) {
	var (
		state    *C.char
		err_kind *C.char
		err_msg  *C.char
	)
	rc := C.nispor_net_state_retrieve(&state, &err_kind, &err_msg)
	defer func() {
		C.nispor_net_state_free(state)
		C.nispor_err_kind_free(err_kind)
		C.nispor_err_msg_free(err_msg)
	}()
	if rc != 0 {
		return "", fmt.Errorf("failed retrieving nisport net state with rc: %d", rc)
	}
	return C.GoString(state), nil
}
