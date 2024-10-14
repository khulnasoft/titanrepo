package auth

import (
	"github.com/khulnasoft/titanrepo/cli/internal/cmdutil"
	"github.com/khulnasoft/titanrepo/cli/internal/util"

	"github.com/spf13/cobra"
)

// UnlinkCmd returns the Cobra unlink command
func UnlinkCmd(helper *cmdutil.Helper) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "unlink",
		Short: "Unlink the current directory from your Khulnasoft organization and disable Remote Caching",
		RunE: func(cmd *cobra.Command, args []string) error {
			base, err := helper.GetCmdBase(cmd.Flags())
			if err != nil {
				return err
			}
			if err := base.RepoConfig.Delete(); err != nil {
				base.LogError("could not unlink. Something went wrong: %w", err)
				return err
			}

			base.UI.Output(util.Sprintf("${GREY}> Disabled Remote Caching${RESET}"))

			return nil
		},
	}

	return cmd
}
