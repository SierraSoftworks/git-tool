package registry

import (
	"context"
	"strings"
	"time"

	"github.com/go-yaml/yaml"
	"github.com/google/go-github/v26/github"
)

type githubSource struct{}

// GitHub acts as a source of configuration entries used by Git-Tool.
func GitHub() Source {
	return &githubSource{}
}

func (s *githubSource) GetEntries() ([]Entry, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	cl := github.NewClient(nil)

	tree, _, err := cl.Git.GetTree(ctx, "sierrasoftworks", "git-tool", "master", true)
	if err != nil {
		return nil, err
	}

	entries := []Entry{}
	for _, entry := range tree.Entries {
		if strings.HasPrefix(entry.GetPath(), "registry/") && entry.GetType() == "blob" {
			data := entry.GetContent()
			if data != "" {
				parsedEntry := Entry{}
				if yaml.Unmarshal([]byte(data), &parsedEntry) == nil {
					entries = append(entries, parsedEntry)
				}
			}
		}
	}

	return entries, nil
}
