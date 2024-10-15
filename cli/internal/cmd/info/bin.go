package info

import (
	"os"

	"github.com/khulnasoft/titanrepo/cli/internal/cmdutil"

	"github.com/spf13/cobra"
)

// BinCmd returns the Cobra bin command
func BinCmd(helper *cmdutil.Helper) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "bin",
		Short: "Get the path to the Titan binary",
		RunE: func(cmd *cobra.Command, args []string) error {
			base, err := helper.GetCmdBase(cmd.Flags())
			if err != nil {
				return err
			}
			path, err := os.Executable()
			if err != nil {
				base.LogError("could not get path to titan binary: %w", err)
				return err
			}

			base.UI.Output(path)

			return nil
		},
	}

	return cmd
}
