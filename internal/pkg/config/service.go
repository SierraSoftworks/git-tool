package config

import (
	"github.com/pkg/errors"
	"bytes"
	"text/template"
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

type service struct {
	DomainField string `json:"domain" yaml:"domain"`

	WebsiteTemplate string `json:"website" yaml:"website"`
	HttpUrlTemplate string `json:"httpUrl" yaml:"httpUrl"`
	GitUrlTemplate string `json:"gitUrl" yaml:"gitUrl"`

	DirectoryGlobField string `json:"pattern" yaml:"pattern"`
}

func (s *service) Domain() string {
	return s.DomainField
}

func (s *service) DirectoryGlob() string {
	return s.DirectoryGlobField
}

func (s *service) Website(r models.Repo) string {
	return s.getTemplateSafe(s.WebsiteTemplate, r, "")
}

func (s *service) HttpURL(r models.Repo) string {
	return s.getTemplateSafe(s.HttpUrlTemplate, r, "")
}

func (s *service) GitURL(r models.Repo) string {
	return s.getTemplateSafe(s.GitUrlTemplate, r, "")
}

func (s *service) getTemplateSafe(tmpl string, r models.Repo, d string) string {
	out, err := s.getTemplate(tmpl, r)
	if err != nil {
		return d
	}

	return out
}

func (s *service) getTemplate(tmpl string, r models.Repo) (string, error) {
	if r.Service() != s {
		return "", errors.New("config: cannot use this service to render a template for a repository belonging to a different service")
	}

	t, err := template.New("gitURL").Parse(tmpl)
	if err != nil {
		return "", err
	}

	buf := bytes.NewBuffer([]byte{})
	if err := t.Execute(buf, struct {
		Service models.Service
		Repo    models.Repo
	}{
		Service: r.Service(),
		Repo:    r,
	}); err != nil {
		return "", err
	}

	return buf.String(), nil
}