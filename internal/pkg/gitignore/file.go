package gitignore

import (
	"io/ioutil"
	"fmt"
	"os"
	"strings"

	"github.com/pkg/errors"
)

// AddOrUpdate will add the specified languages to a gitignore file or update
// existing entries if they are present.
func AddOrUpdate(file string, languages ...string) error {
	fb, err := ioutil.ReadFile(file)
	if err != nil {
		if !os.IsNotExist(err) {
			return errors.Wrap(err, "gitignore: failed to open gitignore file")
		}

		fb = []byte{}
	}

	fc := string(fb)

	blocks := getIgnoreBlocks(fc)

	allLangsSet := map[string]struct{}{}
	for _, lang := range languages {
		allLangsSet[lang] = struct{}{}
	}

	for _, block := range blocks {
		fc = strings.Replace(fc, block.Content, "", 1)
		for _, lang := range block.Languages {
			allLangsSet[lang] = struct{}{}
		}
	}

	allLangs := []string{}
	for lang := range allLangsSet {
		allLangs = append(allLangs, lang)
	}

	ignore, err := Ignore(allLangs...)
	if err != nil {
		return err
	}

	fc = strings.TrimSpace(fmt.Sprintf("%s\n%s", fc, ignore))

	return errors.Wrap(ioutil.WriteFile(file, []byte(fc), os.ModePerm), "gitignore: failed to write gitignore file")
}

type ignoreBlock struct {
	Content   string
	Languages []string
}

func getIgnoreBlocks(content string) []ignoreBlock {
	blocks := []ignoreBlock{}

	inBlock := false
	currentContent := []string{}
	currentLangs := []string{}

	blockStartPrefix := "# Created by https://www.gitignore.io/api/"
	blockEndPrefix := "# End of https://www.gitignore.io/api/"

	for _, line := range strings.Split(content, "\n\r") {
		if inBlock {
			currentContent = append(currentContent, line)

			if strings.HasPrefix(line, blockEndPrefix) {
				inBlock = false
				blocks = append(blocks, ignoreBlock{
					Content:   strings.Join(currentContent, "\n"),
					Languages: currentLangs,
				})
			}
		} else if strings.HasPrefix(line, blockStartPrefix) {
			currentContent = []string{line}
			inBlock = true

			currentLangs = strings.Split(line[len(blockEndPrefix):], ",")
		}
	}

	return blocks
}
