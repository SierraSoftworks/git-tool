package gitignore

import (
	"io/ioutil"
	"os"

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

	managed := ParseSection(fc)
	if managed == nil {
		managed = &ManagedFileSection{
			Languages: languages,
			Prologue:  fc,
			Content:   "",
		}
	} else {
		allLangsSet := map[string]struct{}{}
		for _, lang := range languages {
			allLangsSet[lang] = struct{}{}
		}

		for _, lang := range managed.Languages {
			allLangsSet[lang] = struct{}{}
		}

		managed.Languages = []string{}
		for lang := range allLangsSet {
			managed.Languages = append(managed.Languages, lang)
		}

		managed.Content = ""
	}

	ignore, err := Ignore(managed.Languages...)
	if err != nil {
		return err
	}

	managed.Content = ignore

	fc = managed.String()

	return errors.Wrap(ioutil.WriteFile(file, []byte(fc), os.ModePerm), "gitignore: failed to write gitignore file")
}
