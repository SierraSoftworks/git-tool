package test

import (
	"log"
	"os"
	"path/filepath"
)

// GetProjectRoot will fetch the root directory of a Go project
// from the current working directory.
func GetProjectRoot() string {
	cwd, err := os.Getwd()
	if err != nil {
		log.Fatal(err)
	}

	for cwd != "/" && cwd != "." && cwd != "" {
		_, err := os.Stat(filepath.Join(cwd, "go.mod"))
		if err == nil || os.IsExist(err) {
			return cwd
		}

		cwd = filepath.Dir(cwd)
	}

	return "."
}

// GetTestPath will return the path to the $project/test directory
// holding test tools and data files.
func GetTestPath() string {
	return filepath.Join(GetProjectRoot(), "test")
}

// GetTestDataPath will return the path to a specific test data file
func GetTestDataPath(file ...string) string {
	return filepath.Join(append([]string{GetTestPath(), "data"}, file...)...)
}