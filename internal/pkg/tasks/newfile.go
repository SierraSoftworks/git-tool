package tasks

import (
	"io/ioutil"
	"os"
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
)

// NewFile is responsible for writing a new file.
func NewFile(path string, content []byte) Task {
	return &newFile{
		Path:    path,
		Content: content,
	}
}

// newFile is responsible for writing content to a file.
type newFile struct {
	Path    string
	Content []byte
}

// ApplyRepo runs the task against a repository
func (t *newFile) ApplyRepo(r models.Repo) error {
	err := ioutil.WriteFile(filepath.Join(r.Path(), t.Path), t.Content, os.ModePerm)
	if err != nil {
		return errors.Wrap(err, "repo: unable to write file")
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *newFile) ApplyScratchpad(r models.Scratchpad) error {
	err := ioutil.WriteFile(filepath.Join(r.Path(), t.Path), t.Content, os.ModePerm)
	if err != nil {
		return errors.Wrap(err, "repo: unable to write file")
	}

	return nil
}
