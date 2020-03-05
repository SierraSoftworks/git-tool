package app

import (
	"fmt"
	"time"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"

	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

var openScratchCommand = &cli.Command{
	Name: "scratch",
	Aliases: []string{
		"s",
	},
	Usage:     "Opens your scratch space for the current week.",
	ArgsUsage: "[app] [week]",
	Flags:     []cli.Flag{},
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/scratch")
		defer tracing.Exit()

		args := c.Args().Slice()

		app := di.GetConfig().GetApp(c.Args().First())
		if app == nil {
			app = di.GetConfig().GetDefaultApp()
		} else {
			args = c.Args().Tail()
		}

		if app == nil && c.NArg() > 0 {
			return errors.Errorf("usage: no app called %s in your config", c.Args().First())
		} else if app == nil {
			return errors.Errorf("usage: no apps in your config")
		}

		logrus.WithField("app", app.Name()).Debug("Found matching app configuration")

		name := ""
		if len(args) > 0 {
			name = args[0]
		}

		if name == "" {
			year, week := time.Now().ISOWeek()
			name = fmt.Sprintf("%dw%d", year, week)
		}

		r, err := di.GetMapper().GetScratchpad(name)
		if err != nil {
			return err
		}

		if !r.Exists() {
			tracing.Transition("/app/command/scratch/create")
			if err := di.GetInitializer().CreateScratchpad(r); err != nil {
				return err
			}
		}

		tracing.Transition("/app/command/scratch/run")
		cmd, err := app.GetCmd(r)
		if err != nil {
			return err
		}

		return di.GetLauncher().Run(cmd)
	},
	BashComplete: func(c *cli.Context) {
		tracing.Enter("/app/complete/scratch")
		defer tracing.Exit()

		cmp := autocomplete.NewCompleter(c.String("bash-completion-filter"))

		if c.NArg() == 0 {
			cmp.Apps()
		}

		if app := di.GetConfig().GetApp(c.Args().First()); app != nil {
			cmp.AllScratchpads()
		}
	},
}
