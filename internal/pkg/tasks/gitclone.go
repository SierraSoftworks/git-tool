package tasks

import (
	"os/exec"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"gopkg.in/src-d/go-git.v4"
)

// GitClone is responsible for running the equivalent of a `git clone` for a
// repository.
func GitClone() Task {
	return &gitClone{}
}

// gitClone is responsible for running the equivalent of a `git clone` for a
// repository.
type gitClone struct {
}

// ApplyRepo runs the task against a repository
func (t *gitClone) ApplyRepo(r models.Repo) error {
	if r.Exists() {
		return nil
	}

	if di.GetConfig().GetFeatures().UseNativeClone() {
		return t.cloneNative(r)
	}

	return t.cloneInternal(r)
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitClone) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}

func (t *gitClone) cloneNative(r models.Repo) error {
	url := r.GitURL()
	if di.GetConfig().GetFeatures().UseHttpTransport() {
		url = r.HttpURL()
	}

	cmd := exec.Command(
		"git",
		"clone",
		"--recurse-submodules",
		url,
		r.Path(),
	)

	return di.GetLauncher().Run(cmd)
}

func (t *gitClone) cloneInternal(r models.Repo) error {
	url := r.GitURL()
	if di.GetConfig().GetFeatures().UseHttpTransport() {
		url = r.HttpURL()
	}

	_, err := git.PlainClone(r.Path(), false, &git.CloneOptions{
		URL:               url,
		RecurseSubmodules: git.DefaultSubmoduleRecursionDepth,
		Tags:              git.AllTags,
		RemoteName:        "origin",
	})

	if err != nil {
		return errors.Wrap(err, "repo: unable to clone remote repository")
	}

	return nil
}
