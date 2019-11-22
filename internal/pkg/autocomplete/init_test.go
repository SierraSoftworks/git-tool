package autocomplete_test

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("Init", func() {
	Describe("GetInitScript()", func() {
		cases := []struct {
			Shell    string
			HasValue bool
		}{
			{"powershell", true},
			{"bash", true},
			{"zsh", true},
			{"cmd", false},
			{"fish", false},
		}

		for _, tc := range cases {
			shell := tc.Shell
			expected := tc.HasValue

			Context(fmt.Sprintf("GetInitScript('%s') -> %v", tc.Shell, tc.HasValue), func() {
				if expected {
					It("Should have an init script", func() {
						Expect(autocomplete.GetInitScript(shell)).ToNot(BeEmpty())
					})
				} else {
					It("Should not have an init script", func() {
						Expect(autocomplete.GetInitScript(shell)).To(BeEmpty())
					})
				}
			})
		}
	})

	Describe("GetInitScriptFull()", func() {
		cases := []struct {
			Shell    string
			HasValue bool
		}{
			{"powershell", true},
			{"bash", true},
			{"zsh", true},
			{"cmd", false},
			{"fish", false},
		}

		for _, tc := range cases {
			shell := tc.Shell
			expected := tc.HasValue

			Context(fmt.Sprintf("GetFullInitScript('%s') -> %v", tc.Shell, tc.HasValue), func() {
				if expected {
					It("Should have an init script", func() {
						Expect(autocomplete.GetInitScript(shell)).ToNot(BeEmpty())
					})
				} else {
					It("Should not have an init script", func() {
						Expect(autocomplete.GetInitScript(shell)).To(BeEmpty())
					})
				}
			})
		}
	})

	Describe("GetInitScriptShells()", func() {
		cases := []string{
			"powershell",
			"bash",
			"zsh",
		}

		for _, tc := range cases {
			shell := tc

			It(fmt.Sprintf("Should contain %s", shell), func() {
				Expect(autocomplete.GetInitScriptShells()).To(ContainElement(shell))
			})
		}
	})
})
