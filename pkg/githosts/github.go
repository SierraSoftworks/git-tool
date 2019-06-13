package githosts

import (
	"strings"
	"context"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/google/go-github/v26/github"
	"golang.org/x/oauth2"
)

func init() {
	RegisterHost(&githubHost{})
}

type githubHost struct {}

func (h *githubHost) Handles(s models.Service) bool {
	return s.Domain() == "github.com"
}

func (h *githubHost) HasRepo(r models.Repo) (bool, error) {
	ctx := context.Background()

	cl, err := h.getGithubClient(ctx)
	if err != nil {
		return false, err
	}

	if _, res, err := cl.Repositories.Get(ctx, r.Namespace(), r.Name()); err != nil {
		if res.StatusCode == 404 {
			return false, nil
		}

		return false, err
	}

	return true, nil
}

func (h *githubHost) CreateRepo(r models.Repo) error {
	ctx := context.Background()

	cl, err := h.getGithubClient(ctx)
	if err != nil {
		return err
	}

	ghr := &github.Repository{
		Name:    github.String(r.Name()),
		Private: github.Bool(true),
	}

	org := r.Namespace()

	usr, _, err := cl.Users.Get(ctx, "")
	if err != nil {
		return err
	}

	if strings.ToLower(usr.GetLogin()) == strings.ToLower(org) {
		org = ""
	}

	_, _, err = cl.Repositories.Create(ctx, org, ghr)
	return err
}

func (h *githubHost) getGithubClient(ctx context.Context) (*github.Client, error) {
	creds, err := GetCredentials("github.com")
	if err != nil {
		return nil, err
	}

	if creds == "" {
		return github.NewClient(nil), nil
	}

	ts := oauth2.StaticTokenSource(&oauth2.Token{
		AccessToken: creds,
	})
	tc := oauth2.NewClient(ctx, ts)

	return github.NewClient(tc), nil
}