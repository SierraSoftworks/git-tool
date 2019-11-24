package registry

import (
	"io/ioutil"
	"os"
	"path/filepath"

	"github.com/go-yaml/yaml"
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

func (s *filesystemSource) GetEntries() ([]Entry, error) {

	entries := []Entry{}
	err := filepath.Walk(s.Path, func(path string, info os.FileInfo, err error) error {
		if info.IsDir() {
			return nil
		}

		if filepath.Ext(path) != ".yaml" {
			return nil
		}

		data, err := ioutil.ReadFile(path)
		if err != nil {
			return err
		}

		parsedEntry := Entry{}
		err = yaml.Unmarshal([]byte(data), &parsedEntry)
		if err != nil {
			return err
		}

		entries = append(entries, parsedEntry)

		return nil
	})

	if err != nil {
		return nil, err
	}

	return entries, nil
}
