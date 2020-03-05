package app_test

import (
	"fmt"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGitIgnore(t *testing.T) {
	cmd := "ignore"

	/*----- Setup -----*/

	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd), "it should not return an error") {
			assert.Contains(t, out.GetOperations(), " - csharp\n", "it should print out a list of valid languages")
		}
	})

	t.Run("gt "+cmd+" go", func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd, "go"), "it should not return an error") {
			assert.Len(t, out.GetOperations(), 1, "it should print only the ignore file")
			assert.Contains(t, out.GetOperations()[0], ".exe~", "it should print the ignore file itself")
		}
	})

	t.Run("gt "+cmd+" go node", func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd, "go", "node"), "it should not return an error") {
			assert.Len(t, out.GetOperations(), 1, "it should print only the ignore file")
			assert.Contains(t, out.GetOperations()[0], ".exe~", "it should print the ignore file itself")
			assert.Contains(t, out.GetOperations()[0], "node_modules", "it should merge the ignore files")
		}
	})

	t.Run("Auto Completion", func(t *testing.T) {
		t.Run("App-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", "gt"), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), cmd+"\n", "it should print the command name")
		})

		t.Run("Command-Level", func(t *testing.T) {
			out.Reset()
			require.NoError(t, runApp("complete", fmt.Sprintf("gt %s ", cmd)), "no error should be thrown")

			assert.Contains(t, out.GetOperations(), "csharp\n", "it should print out a list of valid languages")
		})
	})
}
