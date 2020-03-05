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

func TestApps(t *testing.T) {
	cmd := "apps"

	/*----- Setup -----*/

	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

	/*----- Tests -----*/

	require.NotNil(t, app.NewApp().Command(cmd), "the command should be registered with the app")

	t.Run("gt "+cmd, func(t *testing.T) {
		out.Reset()
		if assert.NoError(t, runApp(cmd), "it should not return an error") {
			assert.Len(t, out.GetOperations(), len(di.GetConfig().GetApps()), "it should print out every configured app")

			for _, app := range di.GetConfig().GetApps() {
				assert.Containsf(t, out.GetOperations(), app.Name()+"\n", "it should print out the '%s' app", app.Name())
			}
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

			assert.Empty(t, out.GetOperations(), "it should not print any completion suggestions")
		})
	})
}
