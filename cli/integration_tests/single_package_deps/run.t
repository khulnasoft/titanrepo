Setup
  $ . ${TESTDIR}/../setup.sh
  $ . ${TESTDIR}/setup.sh $(pwd)

Check
  $ ${TITAN} run test --single-package
  \xe2\x80\xa2 Running test (esc)
  \xe2\x80\xa2 Remote caching disabled (esc)
  build: cache miss, executing 6dab35f71988610e
  build: 
  build: > build
  build: > echo 'building' > foo
  build: 
  test: cache miss, executing a4039f6c1959b8db
  test: 
  test: > test
  test: > [[ ( -f foo ) && $(cat foo) == 'building' ]]
  test: 
  
   Tasks:    2 successful, 2 total
  Cached:    0 cached, 2 total
    Time:\s*[\.0-9]+m?s  (re)
  
Run a second time, verify caching works because there is a config
  $ ${TITAN} run test --single-package
  \xe2\x80\xa2 Running test (esc)
  \xe2\x80\xa2 Remote caching disabled (esc)
  build: cache hit, replaying output 6dab35f71988610e
  build: 
  build: > build
  build: > echo 'building' > foo
  build: 
  test: cache hit, replaying output a4039f6c1959b8db
  test: 
  test: > test
  test: > [[ ( -f foo ) && $(cat foo) == 'building' ]]
  test: 
  
   Tasks:    2 successful, 2 total
  Cached:    2 cached, 2 total
    Time:\s*[\.0-9]+m?s >>> FULL TITAN (re)
  