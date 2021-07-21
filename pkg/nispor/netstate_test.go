package nispor

import (
	"bytes"
	"fmt"
	"os/exec"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestRetrieveNetStateIfaces(t *testing.T) {
	npcCmd := exec.Command("npc", "iface", "-j")
	expectedNetState := &bytes.Buffer{}
	npcCmd.Stdout = expectedNetState
	err := npcCmd.Run()
	assert.NoError(t, err, "should succeed calling npc")

	obtainedNetState, err := RetrieveNetStateJSON()
	assert.NoError(t, err, "should succeed calling retrieve_net_state c binding")
	assert.NotEmpty(t, obtainedNetState, "net state should not be empty")
	fmt.Println(obtainedNetState)
	assert.JSONEq(t, expectedNetState.String(), obtainedNetState, "net state should be equal to the 'npc ifaces -j' command output")
}
