package login

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/khulnasoft/titanrepo/cli/internal/client"
	"github.com/khulnasoft/titanrepo/cli/internal/cmdutil"
	"github.com/khulnasoft/titanrepo/cli/internal/fs"
	"github.com/khulnasoft/titanrepo/cli/internal/ui"
	"github.com/khulnasoft/titanrepo/cli/internal/util"
	"github.com/khulnasoft/titanrepo/cli/internal/util/browser"
	"github.com/pkg/errors"
	"github.com/spf13/cobra"

	"github.com/AlecAivazis/survey/v2"
	"github.com/fatih/color"
	"github.com/mitchellh/go-homedir"
)

type link struct {
	base                *cmdutil.CmdBase
	modifyGitIgnore     bool
	apiClient           linkAPIClient // separate from base to allow testing
	promptSetup         func(location string) (bool, error)
	promptTeam          func(teams []string) (string, error)
	promptEnableCaching func() (bool, error)
	openBrowser         func(url string) error
}

type linkAPIClient interface {
	HasUser() bool
	GetTeams() (*client.TeamsResponse, error)
	GetUser() (*client.UserResponse, error)
	SetTeamID(teamID string)
	GetCachingStatus() (util.CachingStatus, error)
}

// NewLinkCommand returns the cobra subcommand for titan link
func NewLinkCommand(helper *cmdutil.Helper) *cobra.Command {
	return getCmd(helper)
}

func getCmd(helper *cmdutil.Helper) *cobra.Command {
	var dontModifyGitIgnore bool
	cmd := &cobra.Command{
		Use:           "link",
		Short:         "Link your local directory to a Khulnasoft organization and enable remote caching.",
		SilenceUsage:  true,
		SilenceErrors: true,
		RunE: func(cmd *cobra.Command, args []string) error {
			base, err := helper.GetCmdBase(cmd.Flags())
			if err != nil {
				return err
			}
			link := &link{
				base:                base,
				modifyGitIgnore:     !dontModifyGitIgnore,
				apiClient:           base.APIClient,
				promptSetup:         promptSetup,
				promptTeam:          promptTeam,
				promptEnableCaching: promptEnableCaching,
				openBrowser:         browser.OpenBrowser,
			}
			err = link.run()
			if err != nil {
				if errors.Is(err, errUserCanceled) {
					base.UI.Info("Canceled. Titanrepo not set up.")
				} else if errors.Is(err, errTryAfterEnable) || errors.Is(err, errNeedCachingEnabled) || errors.Is(err, errOverage) {
					base.UI.Info("Remote Caching not enabled. Please run 'titan login' again after Remote Caching has been enabled")
				} else {
					link.logError(err)
				}
				return err
			}
			return nil
		},
	}
	cmd.Flags().BoolVar(&dontModifyGitIgnore, "no-gitignore", false, "Do not create or modify .gitignore (default false)")
	return cmd
}

var errUserCanceled = errors.New("canceled")

func (l *link) run() error {
	dir, err := homedir.Dir()
	if err != nil {
		return fmt.Errorf("could not find home directory.\n%w", err)
	}
	l.base.UI.Info(">>> Remote Caching")
	l.base.UI.Info("")
	l.base.UI.Info("  Remote Caching shares your cached Titanrepo task outputs and logs across")
	l.base.UI.Info("  all your team’s Khulnasoft projects. It also can share outputs")
	l.base.UI.Info("  with other services that enable Remote Caching, like CI/CD systems.")
	l.base.UI.Info("  This results in faster build times and deployments for your team.")
	l.base.UI.Info(util.Sprintf("  For more info, see ${UNDERLINE}https://titan.khulnasoft.com/docs/core-concepts/remote-caching${RESET}"))
	l.base.UI.Info("")
	currentDir, err := filepath.Abs(".")
	if err != nil {
		return fmt.Errorf("could figure out file path.\n%w", err)
	}
	repoLocation := strings.Replace(currentDir, dir, "~", 1)
	shouldSetup, err := l.promptSetup(repoLocation)
	if err != nil {
		return err
	}
	if !shouldSetup {
		return errUserCanceled
	}

	if !l.apiClient.HasUser() {
		return fmt.Errorf(util.Sprintf("User not found. Please login to Titanrepo first by running ${BOLD}`npx titan login`${RESET}."))
	}

	teamsResponse, err := l.apiClient.GetTeams()
	if err != nil {
		return fmt.Errorf("could not get team information.\n%w", err)
	}
	userResponse, err := l.apiClient.GetUser()
	if err != nil {
		return fmt.Errorf("could not get user information.\n%w", err)
	}

	// Gather team options
	teamOptions := make([]string, len(teamsResponse.Teams)+1)
	nameWithFallback := userResponse.User.Name
	if nameWithFallback == "" {
		nameWithFallback = userResponse.User.Username
	}
	teamOptions[0] = nameWithFallback
	for i, team := range teamsResponse.Teams {
		teamOptions[i+1] = team.Name
	}

	chosenTeamName, err := l.promptTeam(teamOptions)
	if err != nil {
		return err
	}
	if chosenTeamName == "" {
		return errUserCanceled
	}
	isUser := (chosenTeamName == userResponse.User.Name) || (chosenTeamName == userResponse.User.Username)
	var chosenTeam client.Team
	var teamID string
	if isUser {
		teamID = userResponse.User.ID
	} else {
		for _, team := range teamsResponse.Teams {
			if team.Name == chosenTeamName {
				chosenTeam = team
				break
			}
		}
		teamID = chosenTeam.ID
	}
	l.apiClient.SetTeamID(teamID)

	cachingStatus, err := l.apiClient.GetCachingStatus()
	if err != nil {
		return err
	}
	switch cachingStatus {
	case util.CachingStatusDisabled:
		if isUser || chosenTeam.IsOwner() {
			shouldEnable, err := l.promptEnableCaching()
			if err != nil {
				return err
			}
			if shouldEnable {
				var url string
				if isUser {
					url = "https://khulnasoft.com/account/billing"
				} else {
					url = fmt.Sprintf("https://khulnasoft.com/teams/%v/settings/billing", chosenTeam.Slug)
				}
				err = l.openBrowser(url)
				if err != nil {
					l.base.UI.Warn(fmt.Sprintf("Failed to open browser. Please visit %v to enable Remote Caching", url))
				} else {
					l.base.UI.Info(fmt.Sprintf("Visit %v in your browser to enable Remote Caching", url))
				}
				return errTryAfterEnable
			}
		}
		return errNeedCachingEnabled
	case util.CachingStatusOverLimit:
		return errOverage
	case util.CachingStatusPaused:
		return errPaused
	case util.CachingStatusEnabled:
	default:
	}

	fs.EnsureDir(filepath.Join(".titan", "config.json"))
	err = l.base.RepoConfig.SetTeamID(teamID)
	if err != nil {
		return fmt.Errorf("could not link current directory to team/user.\n%w", err)
	}

	if l.modifyGitIgnore {
		if err := l.addTitanToGitignore(); err != nil {
			return err
		}
	}

	l.base.UI.Info("")
	l.base.UI.Info(util.Sprintf("%s${RESET} Titanrepo CLI authorized for ${BOLD}%s${RESET}", ui.Rainbow(">>> Success!"), chosenTeamName))
	l.base.UI.Info("")
	l.base.UI.Info(util.Sprintf("${GREY}To disable Remote Caching, run `npx titan unlink`${RESET}"))
	l.base.UI.Info("")
	return nil
}

// logError logs an error and outputs it to the UI.
func (l *link) logError(err error) {
	l.base.Logger.Error(fmt.Sprintf("error: %v", err))
	l.base.UI.Error(fmt.Sprintf("%s%s", ui.ERROR_PREFIX, color.RedString(" %v", err)))
}

func promptSetup(location string) (bool, error) {
	shouldSetup := true
	err := survey.AskOne(
		&survey.Confirm{
			Default: true,
			Message: util.Sprintf("Would you like to enable Remote Caching for ${CYAN}${BOLD}\"%s\"${RESET}?", location),
		},
		&shouldSetup, survey.WithValidator(survey.Required),
		survey.WithIcons(func(icons *survey.IconSet) {
			// for more information on formatting the icons, see here: https://github.com/mgutz/ansi#style-format
			icons.Question.Format = "gray+hb"
		}))
	if err != nil {
		return false, err
	}
	return shouldSetup, nil
}

func (l *link) addTitanToGitignore() error {
	gitignorePath := l.base.RepoRoot.Join(".gitignore")

	if !gitignorePath.FileExists() {
		err := gitignorePath.WriteFile([]byte(".titan\n"), 0644)
		if err != nil {
			return fmt.Errorf("could not create .gitignore.\n%w", err)
		}
		return nil
	}

	gitignoreBytes, err := gitignorePath.ReadFile()
	if err != nil {
		return fmt.Errorf("could not find or update .gitignore.\n%w", err)
	}

	hasTitan := false
	gitignoreContents := string(gitignoreBytes)
	gitignoreLines := strings.Split(gitignoreContents, "\n")

	for _, line := range gitignoreLines {
		if strings.TrimSpace(line) == ".titan" {
			hasTitan = true
			break
		}
	}

	if !hasTitan {
		gitignore, err := gitignorePath.OpenFile(os.O_APPEND|os.O_WRONLY, 0644)
		if err != nil {
			return fmt.Errorf("could not find or update .gitignore.\n%w", err)
		}

		// if the file doesn't end in a newline, we add one
		if !strings.HasSuffix(gitignoreContents, "\n") {
			if _, err := gitignore.WriteString("\n"); err != nil {
				return fmt.Errorf("could not find or update .gitignore.\n%w", err)
			}
		}

		if _, err := gitignore.WriteString(".titan\n"); err != nil {
			return fmt.Errorf("could not find or update .gitignore.\n%w", err)
		}
	}

	return nil
}

func promptTeam(teams []string) (string, error) {
	chosenTeamName := ""
	err := survey.AskOne(
		&survey.Select{
			Message: "Which Khulnasoft scope (and Remote Cache) do you want associate with this Titanrepo? ",
			Options: teams,
		},
		&chosenTeamName,
		survey.WithValidator(survey.Required),
		survey.WithIcons(func(icons *survey.IconSet) {
			// for more information on formatting the icons, see here: https://github.com/mgutz/ansi#style-format
			icons.Question.Format = "gray+hb"
		}))
	if err != nil {
		return "", err
	}
	return chosenTeamName, nil
}
