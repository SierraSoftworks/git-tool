package tasks_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSequence(t *testing.T) {
	r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
	sp := repo.NewScratchpad("2019w15")

	t.Run("No Tasks", func(t *testing.T) {
		seq := tasks.Sequence()
		require.NotNil(t, seq, "it should return a task")

		assert.NoError(t, seq.ApplyRepo(r), "it should not return an error when applied to a repo")
		assert.NoError(t, seq.ApplyScratchpad(sp), "it should not return an error when applied to a scratchpad")
	})

	t.Run("With Tasks", func(t *testing.T) {
		t.Run("On a Repo", func(t *testing.T) {
			var (
				firstCalled  = false
				secondCalled = false
			)

			seq := tasks.Sequence(&TestTask{
				OnRepo: func(rr models.Repo) error {
					firstCalled = true
					assert.False(t, secondCalled, "the second task should not be called before the first")
					assert.Equal(t, r, rr, "the correct repository should be passed to the task")
					return nil
				},
			}, &TestTask{
				OnRepo: func(rr models.Repo) error {
					secondCalled = true
					assert.True(t, firstCalled, "the first task should be called before the second")
					assert.Equal(t, r, rr, "the correct repository should be passed to the task")
					return nil
				},
			})

			assert.NoError(t, seq.ApplyRepo(r), "it should not return an error")
			assert.True(t, firstCalled, "it should have called the first task")
			assert.True(t, secondCalled, "it should have called the second task")
		})

		t.Run("On a Scratchpad", func(t *testing.T) {
			var (
				firstCalled  = false
				secondCalled = false
			)

			seq := tasks.Sequence(&TestTask{
				OnScratchpad: func(ssp models.Scratchpad) error {
					firstCalled = true
					assert.False(t, secondCalled, "the second task should not be called before the first")
					assert.Equal(t, sp, ssp, "the correct scratchpad should be passed to the task")
					return nil
				},
			}, &TestTask{
				OnScratchpad: func(ssp models.Scratchpad) error {
					secondCalled = true
					assert.True(t, firstCalled, "the first task should be called before the second")
					assert.Equal(t, sp, ssp, "the correct scratchpad should be passed to the task")
					return nil
				},
			})

			assert.NoError(t, seq.ApplyScratchpad(sp), "it should not return an error")
			assert.True(t, firstCalled, "it should have called the first task")
			assert.True(t, secondCalled, "it should have called the second task")
		})
	})
}
