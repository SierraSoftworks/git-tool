package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/pkg/errors"
	"github.com/urfave/cli"
)

var newRepoCommand = cli.Command{
	Name: "new",
	Aliases: []string{
		"create",
		"n",
	},
	Usage:     "Creates a new repository with the provided name.",
	ArgsUsage: "repo",
	Action: func(c *cli.Context) error {
		if c.NArg() == 0 {
			return errors.New("no repository specified")
		}

		r, err := di.GetMapper().GetRepo(c.Args().First())
		if err != nil {
			return err
		}

		if r == nil {
			return errors.New("not a valid repository name")
		}

		init := di.GetInitializer()
		err = init.CreateRepository(r)
		if err != nil {
			return err
		}

		return nil
	},
	BashComplete: func(c *cli.Context) {
		if c.NArg() > 0 {
			return
		}

		cmp := autocomplete.NewCompleter(c.GlobalString("bash-completion-filter"))

		cmp.DefaultServiceNamespaces()
		cmp.AllServiceNamespaces()
	},
}
