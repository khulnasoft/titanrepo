Setup
  $ . ${TESTDIR}/setup.sh
  $ . ${TESTDIR}/logged_in.sh

Logout while logged in
  $ ${TITAN} logout
  >>> Logged out

Logout while logged out
  $ ${TITAN} logout
  >>> Logged out

