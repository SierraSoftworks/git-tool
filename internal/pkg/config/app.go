package config

import (
	"bytes"
	"os"
	"os/exec"
	"text/template"

	"github.com/SierraSoftworks/git-tool/pkg/models"

	"github.com/pkg/errors"
)

// An app is something which may be executed within the context of your
// repository.
type app struct {
	NameField   string   `yaml:"name"`
	CommandLine string   `yaml:"command"`
	Arguments   []string `yaml:"args,omitempty"`
	Environment []string `yaml:"environment,omitempty"`
}

func (a *app) Name() string {
	return a.NameField
}

// GetCmd will fetch the *exec.Cmd used to start this application within
// the context of a specific service and repository.
func (a *app) GetCmd(r models.Target) (*exec.Cmd, error) {
	ctx := r.TemplateContext()

	args := make([]string, len(a.Arguments))

	for i, arg := range a.Arguments {
		at, err := a.getTemplate(arg, ctx)
		if err != nil {
			return nil, errors.Wrap(err, "config: failed to construct application command line")
		}

		args[i] = at
	}

	env := make([]string, len(a.Environment))

	for i, arg := range a.Environment {
		at, err := a.getTemplate(arg, ctx)
		if err != nil {
			return nil, errors.Wrap(err, "config: failed to construct application environment variables")
		}

		env[i] = at
	}

	cmd := exec.Command(a.CommandLine, args...)

	cmd.Dir = r.Path()
	cmd.Env = append(os.Environ(), env...)

	return cmd, nil
}

func (a *app) getTemplate(tmpl string, ctx interface{}) (string, error) {
	t, err := template.New("gitURL").Parse(tmpl)
	if err != nil {
		return "", err
	}

	buf := bytes.NewBuffer([]byte{})
	if err := t.Execute(buf, ctx); err != nil {
		return "", err
	}

	return buf.String(), nil
}
