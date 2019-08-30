package repo

import (
	"os"
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

// NewScratchpad creates a new scratchpad with the given name
func NewScratchpad(name string) models.Scratchpad {
	return &scratchpad{
		fullName: name,
		path:     filepath.Join(di.GetConfig().ScratchDirectory(), name),
	}
}

// scratchpad describes a repository object with any necessary properties required by
// Git-Tool.
type scratchpad struct {
	fullName string
	path     string
}

func (r *scratchpad) Name() string {
	return r.fullName
}

func (r *scratchpad) Path() string {
	return r.path
}

func (r *scratchpad) Exists() bool {
	s, err := os.Stat(r.path)
	if err != nil && os.IsNotExist(err) {
		return false
	}

	if err != nil {
		return true
	}

	return s.IsDir()
}

func (r *scratchpad) TemplateContext() *models.TemplateContext {
	return &models.TemplateContext{
		Target:     r,
		Scratchpad: r,
	}
}

type scratchpadService struct{}

func (s *scratchpadService) Domain() string {
	return "scratch"
}

func (s *scratchpadService) DirectoryGlob() string {
	return "*"
}

func (s *scratchpadService) Website(r models.Repo) string {
	return ""
}

func (s *scratchpadService) GitURL(r models.Repo) string {
	return ""
}

func (s *scratchpadService) HttpURL(r models.Repo) string {
	return ""
}
