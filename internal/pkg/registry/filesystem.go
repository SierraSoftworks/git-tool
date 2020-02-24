package registry

import (
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"

	"github.com/go-yaml/yaml"
	"github.com/pkg/errors"
)

type filesystemSource struct {
	Path string
}

// FileSystem creates a configuration source which uses a filesystem directory to hold
// entries.
func FileSystem(path string) Source {
	return &filesystemSource{
		Path: path,
	}
}

func (s *filesystemSource) GetEntries() ([]string, error) {
	entries := []string{}
	err := filepath.Walk(s.Path, func(path string, info os.FileInfo, err error) error {
		if info.IsDir() {
			return nil
		}

		if filepath.Ext(path) != ".yaml" {
			return nil
		}

		entry := filepath.ToSlash(strings.Trim(path[len(s.Path):len(path)-len(".yaml")], string(filepath.Separator)))

		entries = append(entries, entry)

		return nil
	})

	if err != nil {
		return nil, err
	}

	return entries, nil
}

func (s *filesystemSource) GetEntry(id string) (*Entry, error) {
	data, err := ioutil.ReadFile(filepath.Join(s.Path, fmt.Sprintf("%s.yaml", filepath.FromSlash(id))))

	if os.IsNotExist(err) {
		return nil, errors.New("registry: could not find entry")
	}

	if err != nil {
		return nil, err
	}

	parsedEntry := Entry{}
	if err := yaml.Unmarshal([]byte(data), &parsedEntry); err != nil {
		return nil, errors.Wrap(err, "registry: unable to parse entry")
	}
	return &parsedEntry, nil
}
