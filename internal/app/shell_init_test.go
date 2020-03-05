package app_test

import (
	"fmt"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestShellInit(t *testing.T) {
	cmd := "shell-init"

	/*----- Setup -----*/

	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd), "it should not return an error") {
			assert.Len(t, out.GetOperations(), len(autocomplete.GetInitScriptShells()), "it should print out the list of shell providers")
		}
	})

	for _, shell := range autocomplete.GetInitScriptShells() {
		shell := shell

		t.Run("gt "+cmd+" "+shell, func(t *testing.T) {
			out.Reset()
			if assert.NoError(t, runApp(cmd, shell), "it should not return an error") {
				assert.Contains(t, out.GetOperations(), autocomplete.GetInitScript(shell), "it should print the init script")
			}
		})

		t.Run("gt "+cmd+" "+shell+" --full", func(t *testing.T) {
			out.Reset()
			if assert.NoError(t, runApp(cmd, shell, "--full"), "it should not return an error") {
				assert.Contains(t, out.GetOperations(), autocomplete.GetFullInitScript(shell), "it should print the full init script")
			}
		})
	}

	t.Run("Auto Completion", func(t *testing.T) {

		t.Run("App-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", "gt"), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), cmd+"\n", "it should print the command name")
		})

		t.Run("Command-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", fmt.Sprintf("gt %s ", cmd)), "no error should be thrown")

			for _, shell := range autocomplete.GetInitScriptShells() {
				assert.Contains(t, out.GetOperations(), shell+"\n", "it should print a completion entry for each shell")
			}
		})
	})
}
