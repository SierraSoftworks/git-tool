package repo

import (
	"os"

	"github.com/SierraSoftworks/git-tool/pkg/models"
)

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
