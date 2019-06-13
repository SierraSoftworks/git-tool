package gitignore

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"net/url"
	"strings"

	"github.com/pkg/errors"
)

// Ignore will fetch a .gitignore file for the provided language(s)
func Ignore(languages ...string) (string, error) {
	if len(languages) == 0 {
		return "", nil
	}

	return get(fmt.Sprintf("/api/%s", strings.Join(languages, ",")))
}

// List gets the list of supported languages available for request
func List() ([]string, error) {
	file, err := get("/api/list")
	if err != nil {
		return nil, err
	}

	langs := []string{}
	for _, row := range strings.Split(file, "\n") {
		langs = append(langs, strings.Split(row, ",")...)
	}

	return langs, nil
}

func get(resource string) (string, error) {
	u, _ := url.Parse("https://gitignore.io")
	u.Path = resource

	req, err := http.NewRequest("GET", u.String(), nil)
	if err != nil {
		return "", errors.Wrap(err, "gitignore: unable to create new web request")
	}

	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return "", errors.Wrap(err, "gitignore: unable to make web request")
	}

	if res.StatusCode != 200 {
		return "", nil
	}

	bs, err := ioutil.ReadAll(res.Body)
	if err != nil {
		return "", errors.Wrap(err, "gitignore: unable to read web response")
	}

	return string(bs), nil
}
