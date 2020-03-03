package tasks

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/plumbing"
)

// GitNewRef is responsible for creating a new Git reference which points at HEAD.
func GitNewRef(name string) Task {
	return &gitNewRef{
		Name: name,
	}
}

// gitCommit is responsible for creating a new git reference which points at HEAD.
type gitNewRef struct {
	Name string
}

// ApplyRepo runs the task against a repository
func (t *gitNewRef) ApplyRepo(r models.Repo) error {
	gr, err := git.PlainOpen(r.Path())
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git repository")
	}

	head, err := gr.Head()
	if err != nil {
		return errors.Wrap(err, "repo: unable to get repo HEAD")
	}

	ref := plumbing.NewHashReference(plumbing.ReferenceName(t.Name), head.Hash())

	logrus.WithField("repo", r).WithField("ref", t.Name).Debugf("Creating new git ref")
	err = gr.Storer.SetReference(ref)

	if err != nil {
		return errors.Wrap(err, "repo: unable to create git ref")
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitNewRef) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
