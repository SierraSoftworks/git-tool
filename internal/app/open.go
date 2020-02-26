package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"

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
		tracing.Enter("/app/command/open")
		defer tracing.Exit()

		args := c.Args()

		app := di.GetConfig().GetApp(c.Args().First())
		if app == nil {
			app = di.GetConfig().GetDefaultApp()
		} else {
			args = cli.Args(c.Args().Tail())
		}

		if app == nil && c.NArg() > 0 {
			return errors.Errorf("usage: no app called %s in your config", c.Args().First())
		} else if app == nil {
			return errors.Errorf("usage: no apps in your config")
		}

		logrus.WithField("app", app.Name()).Debug("Found matching app configuration")

		r, err := di.GetMapper().GetBestRepo(args.First())
		if err != nil {
			return err
		}

		if r == nil && len(args) == 0 {
			return errors.New("usage: no repository specified")
		} else if r == nil {
			return errors.New("usage: could not find repository")
		}

		if !r.Exists() {
			tracing.Transition("/app/command/open/clone")
			init := di.GetInitializer()

			err := init.CloneRepository(r)
			if err != nil {
				logrus.WithError(err).Error("Failed to clone repository")
				return errors.New("usage: repository doesn't exist yet, use 'new' to create it")
			}
		}

		tracing.Transition("/app/command/open/run")

		cmd, err := app.GetCmd(r)
		if err != nil {
			return err
		}

		return di.GetLauncher().Run(cmd)
	},
	BashComplete: func(c *cli.Context) {
		tracing.Enter("/app/complete/open")
		defer tracing.Exit()

		cmp := autocomplete.NewCompleter(c.GlobalString("bash-completion-filter"))

		if c.NArg() == 0 {
			cmp.Apps()
		}

		if app := di.GetConfig().GetApp(c.Args().First()); app != nil {
			cmp.RepoAliases()
			cmp.DefaultServiceRepos()
			cmp.AllServiceRepos()
		}
	},
}
