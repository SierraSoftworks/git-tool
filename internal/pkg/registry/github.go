package registry

import (
	"context"
	"fmt"
	"path"
	"strings"
	"time"

	"github.com/go-yaml/yaml"
	"github.com/google/go-github/v26/github"
	"github.com/pkg/errors"
)

type githubSource struct{}

// GitHub acts as a source of configuration entries used by Git-Tool.
func GitHub() Source {
	return &githubSource{}
}

func (s *githubSource) GetEntries() ([]string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	cl := github.NewClient(nil)

	tree, _, err := cl.Git.GetTree(ctx, "sierrasoftworks", "git-tool", "master", true)
	if err != nil {
		return nil, err
	}

	entries := []string{}
	for _, entry := range tree.Entries {
		if strings.HasPrefix(entry.GetPath(), "registry/") && path.Ext(entry.GetPath()) == ".yaml" && entry.GetType() == "blob" {
			if entry.GetSize() > 0 {
				entry := strings.Trim(entry.GetPath()[len("registry/"):len(entry.GetPath())-len(".yaml")], "/")

				entries = append(entries, entry)
			}
		}
	}

	return entries, nil
}

func (s *githubSource) GetEntry(id string) (*Entry, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	cl := github.NewClient(nil)

	file, _, _, err := cl.Repositories.GetContents(ctx, "sierrasoftworks", "git-tool", path.Join("registry", fmt.Sprintf("%s.yaml", id)), &github.RepositoryContentGetOptions{Ref: "master"})
	if err != nil {
		return nil, errors.Wrap(err, "registry: could not find entry")
	}

	data, err := file.GetContent()
	if err != nil {
		return nil, errors.Wrap(err, "registry: could not decode file content")
	}

	parsedEntry := Entry{}
	if err := yaml.Unmarshal([]byte(data), &parsedEntry); err != nil {
		return nil, errors.Wrap(err, "registry: unable to parse entry")
	}
	return &parsedEntry, nil
}
