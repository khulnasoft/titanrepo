#!/usr/bin/env bash

read -r -d '' CONFIG <<- EOF
{
  "token": "normal-user-token"
}
EOF

USER_CONFIG_HOME=$(mktemp -d -t titan-XXXXXXXXXX)
# duplicate over to XDG var so that titan picks it up
export XDG_CONFIG_HOME=$USER_CONFIG_HOME

mkdir -p $USER_CONFIG_HOME/titanrepo
echo $CONFIG > $USER_CONFIG_HOME/titanrepo/config.json
