package gitignore

import (
	"fmt"
	"strings"
)

type ManagedFileSection struct {
	Prologue  string
	Languages []string
	Content   string
}

func (h *ManagedFileSection) String() string {
	if h.Prologue == "" && h.Content == "" && len(h.Languages) == 0 {
		return ""
	}

	return strings.TrimSpace(strings.Join([]string{
		h.Prologue,
		"## -------- Managed by Git Tool -------- ##",
		"## Add any custom rules above this block ##",
		"## ------------------------------------- ##",
		fmt.Sprintf("## @languages: %s", strings.Join(h.Languages, ",")),
		h.Content,
	}, "\n"))
}

const blockStart = "## -------- Managed by Git Tool -------- ##"

func ParseSection(content string) *ManagedFileSection {
	var section *ManagedFileSection

	lines := strings.Split(content, "\n")
	for i, line := range lines {
		line = strings.TrimSpace(line)

		if section == nil {
			if line == blockStart {
				section = &ManagedFileSection{
					Languages: []string{},
					Prologue:  strings.Join(lines[:i], "\n"),
				}
			} else {
				continue
			}
		} else if !strings.HasPrefix(line, "##") {
			section.Content = strings.Join(lines[i:], "\n")
			break
		}

		switch true {
		case strings.HasPrefix(line, "## @languages:"):
			section.Languages = strings.Split(line[len("## @languages: "):], ",")
			for i, l := range section.Languages {
				section.Languages[i] = strings.TrimSpace(l)
			}

		default:
		}
	}

	return section
}
