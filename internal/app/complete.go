package app

import (
	"strings"

	"github.com/kballard/go-shellquote"
	"github.com/urfave/cli/v2"
)

var completeCommand = &cli.Command{
	Name:  "complete",
	Usage: "Generates autocomplete suggestions for the provided command.",
	Flags: []cli.Flag{
		&cli.IntFlag{
			Name:  "position",
			Usage: "The position of the cursor when the completion is requested",
			Value: -1,
		},
	},
	Hidden: true,
	Action: func(c *cli.Context) error {
		cmd := c.Args().First()
		if cmd == "" {
			return nil
		}

		if c.Int("position") > 0 && c.Int("position") < len(cmd) {
			cmd = cmd[:c.Int("position")]
		}

		filter := ""

		lsi := strings.LastIndex(cmd, " ")
		if lsi > 0 && c.Int("position") <= len(c.Args().First()) {
			filter = cmd[lsi+1:]
			cmd = cmd[:lsi]
		}

		args, err := shellquote.Split(cmd)
		if err != nil {
			return err
		}

		args = append([]string{args[0], "--bash-completion-filter", filter, "--config", c.String("config")}, args[1:]...)

		return c.App.Run(append(args, "--generate-bash-completion"))
	},
}
