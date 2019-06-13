package templates

import (
	"bytes"
	"strings"
	"text/template"
)

func toString(tmpl *template.Template, name string, c interface{}) string {
	buf := bytes.NewBuffer([]byte{})
	tmpl.ExecuteTemplate(buf, name, c)
	return buf.String()
}

func buildTemplates(tmpls map[string]string) *template.Template {
	var t *template.Template

	for name, content := range tmpls {
		if t == nil {
			t = template.Must(template.New(name).Parse(strings.TrimSpace(content)))
		} else {
			t = template.Must(t.New(name).Parse(strings.TrimSpace(content)))
		}
	}

	return t
}
