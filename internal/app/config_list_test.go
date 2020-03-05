package app_test

import (
	"fmt"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestConfigList(t *testing.T) {
	cmd := "config list"

	/*----- Setup -----*/

	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

	/*----- Tests -----*/

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp("config", "list"), "it should not return an error") {
			assert.Greater(t, len(out.GetOperations()), 0, "it should print out every registry entry")

			assert.Contains(t, out.GetOperations(), "apps/bash\n", "it should print out bash app")
			assert.Contains(t, out.GetOperations(), "services/github\n", "it should print out github service")
		}
	})

	t.Run("Auto Completion", func(t *testing.T) {

		t.Run("App-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", "gt", "config"), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), "list\n", "it should print the command name")
		})

		t.Run("Command-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", fmt.Sprintf("gt %s ", cmd)), "no error should be thrown")

			assert.Empty(t, out.GetOperations(), "it should not print any completion suggestions")
		})
	})
}
