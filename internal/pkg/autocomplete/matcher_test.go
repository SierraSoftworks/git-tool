package autocomplete_test

import (
	"fmt"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	. "github.com/smartystreets/goconvey/convey"
)

func TestMatcher(t *testing.T) {
	Convey("Matcher", t, func() {
		Convey("Matches()", func() {
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

				Convey(fmt.Sprintf("Matches('%s', '%s') -> %v", tc.Value, tc.Seq, tc.Match), func() {
					if expected {
						So(autocomplete.Matches(val, seq), ShouldBeTrue)
					} else {
						So(autocomplete.Matches(val, seq), ShouldBeFalse)
					}
				})
			}
		})
	})
}
