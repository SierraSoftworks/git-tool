package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
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
		r, err := getMapper().GetRepo(c.Args().First())
		if err != nil {
			return err
		}

		if r == nil {
			return errors.New("not a valid repository name")
		}

		init := &repo.Initializer{}
		err = init.Init(r)
		if err != nil {
			return err
		}

		return nil
	},
	BashComplete: func(c *cli.Context) {
		if c.NArg() > 0 {
			return
		}

		cmp := autocomplete.NewCompleter(cfg, c.GlobalString("bash-completion-filter"))

		cmp.DefaultServiceNamespaces()
		cmp.AllServiceNamespaces()
	},
}
