package autocomplete_test

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
    . "github.com/onsi/ginkgo"
    . "github.com/onsi/gomega"
)

var _ = Describe("Matcher", func() {
	Describe("Matches()", func() {
		cases := []struct {
			Value string
			Seq   string
			Match bool
		}{
			{"test", "t", true},
			{"test", "x", false},
			{"test", "te", true},
			{"test", "tst", true},
			{"test", "tsts", false},
			{"test", "et", true},
		}

		for _, tc := range cases {
			val := tc.Value
			seq := tc.Seq
			expected := tc.Match

			Context(fmt.Sprintf("Matches('%s', '%s') -> %v", tc.Value, tc.Seq, tc.Match), func() {
				if expected {
					It("Should match", func() {
						Expect(autocomplete.Matches(val, seq)).To(BeTrue())
					})
				} else {
					It("Should not match", func() {
						Expect(autocomplete.Matches(val, seq)).To(BeFalse())
					})
				}
			})
		}
	})
})