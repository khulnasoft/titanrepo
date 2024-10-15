Setup
  $ . ${TESTDIR}/setup.sh

Test help flag
  $ ${TITAN} -h
  The build system that makes ship happen
  
  Usage:
    titan [command]
  
  Available Commands:
    bin         Get the path to the Titan binary
    completion  Generate the autocompletion script for the specified shell
    daemon      Runs the Titanrepo background daemon
    help        Help about any command
    link        Link your local directory to a Khulnasoft organization and enable remote caching.
    login       Login to your Khulnasoft account
    logout      Logout of your Khulnasoft account
    prune       Prepare a subset of your monorepo.
    run         Run tasks across projects in your monorepo
    unlink      Unlink the current directory from your Khulnasoft organization and disable Remote Caching
  
  Flags:
        --api string          Override the endpoint for API calls
        --color               Force color usage in the terminal
        --cpuprofile string   Specify a file to save a cpu profile
        --cwd string          The directory in which to run titan
        --heap string         Specify a file to save a pprof heap profile
    -h, --help                help for titan
        --login string        Override the login endpoint
        --no-color            Suppress color usage in the terminal
        --preflight           When enabled, titan will precede HTTP requests with an OPTIONS request for authorization
        --team string         Set the team slug for API calls
        --token string        Set the auth token for API calls
        --trace string        Specify a file to save a pprof trace
    -v, --verbosity count     verbosity
        --version             version for titan
  
  Use "titan [command] --help" for more information about a command.

  $ ${TITAN} --help
  The build system that makes ship happen
  
  Usage:
    titan [command]
  
  Available Commands:
    bin         Get the path to the Titan binary
    completion  Generate the autocompletion script for the specified shell
    daemon      Runs the Titanrepo background daemon
    help        Help about any command
    link        Link your local directory to a Khulnasoft organization and enable remote caching.
    login       Login to your Khulnasoft account
    logout      Logout of your Khulnasoft account
    prune       Prepare a subset of your monorepo.
    run         Run tasks across projects in your monorepo
    unlink      Unlink the current directory from your Khulnasoft organization and disable Remote Caching
  
  Flags:
        --api string          Override the endpoint for API calls
        --color               Force color usage in the terminal
        --cpuprofile string   Specify a file to save a cpu profile
        --cwd string          The directory in which to run titan
        --heap string         Specify a file to save a pprof heap profile
    -h, --help                help for titan
        --login string        Override the login endpoint
        --no-color            Suppress color usage in the terminal
        --preflight           When enabled, titan will precede HTTP requests with an OPTIONS request for authorization
        --team string         Set the team slug for API calls
        --token string        Set the auth token for API calls
        --trace string        Specify a file to save a pprof trace
    -v, --verbosity count     verbosity
        --version             version for titan
  
  Use "titan [command] --help" for more information about a command.
