package autocomplete_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/stretchr/testify/assert"
)

func TestServices(t *testing.T) {
	out := &mocks.Output{}
	di.SetOutput(out)
	di.SetConfig(config.Default())

	t.Run("With no filter", func(t *testing.T) {
		out.Reset()
		c := autocomplete.NewCompleter("")

		c.Services()
		assert.Contains(t, out.GetOperations(), "github.com\n", "it should print out the default services")
	})

	t.Run("With a matching filter", func(t *testing.T) {
		out.Reset()
		c := autocomplete.NewCompleter("github")

		c.Services()
		assert.Contains(t, out.GetOperations(), "github.com\n", "it should print out the default services")
	})

	t.Run("With a filter that doesn't match", func(t *testing.T) {
		out.Reset()
		c := autocomplete.NewCompleter("nomatch")

		c.Services()
		assert.Empty(t, out.GetOperations(), "it should not print out any services")
	})
}
