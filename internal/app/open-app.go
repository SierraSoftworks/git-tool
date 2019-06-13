package app

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"

	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli"
)

var openAppCommand = cli.Command{
	Name: "open",
	Aliases: []string{
		"run",
		"o",
	},
	Usage:     "Opens the requested repository in a specific command.",
	ArgsUsage: "[app] [repo]",
	Flags:     []cli.Flag{},
	Action: func(c *cli.Context) error {
		args := c.Args()

		app := cfg.GetApp(c.Args().First())
		if app == nil {
			app = cfg.GetDefaultApp()
		} else {
			args = cli.Args(c.Args().Tail())
		}

		if app == nil && c.NArg() > 0 {
			return errors.Errorf("no app called %s in your config", c.Args().First())
		} else if app == nil {
			return errors.Errorf("no apps in your config")
		}

		logrus.WithField("app", app.Name()).Debug("Found matching app configuration")

		r, err := getMapper().GetRepo(args.First())
		if err != nil {
			return err
		}

		if r == nil {
			return errors.New("could not find repository")
		}

		if !r.Exists() {
			init := repo.Initializer{}

			err := init.Clone(r)
			if err != nil {
				return errors.New("repository doesn't exist yet, use 'new' to create it")
			}

			logrus.Info("Cloned repository to your local filesystem")
		}

		cmd, err := app.GetCmd(r)
		if err != nil {
			return err
		}

		cmd.Stdin = os.Stdin
		cmd.Stderr = os.Stderr
		cmd.Stdout = os.Stdout

		return cmd.Run()
	},
	BashComplete: func(c *cli.Context) {
		cmp := autocomplete.NewCompleter(cfg, c.GlobalString("bash-completion-filter"))

		if c.NArg() == 0 {
			cmp.Apps()
		}

		if app := cfg.GetApp(c.Args().First()); app != nil {
			cmp.DefaultServiceRepos()
			cmp.AllServiceRepos()
		}
	},
}
