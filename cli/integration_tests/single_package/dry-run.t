Setup
  $ . ${TESTDIR}/../setup.sh
  $ . ${TESTDIR}/setup.sh $(pwd)

Check
  $ ${TITAN} run build --dry --single-package
  
  Tasks to Run
  build
    Task            = build                  
    Hash            = e1872eef9417d83e       
    Cached (Local)  = false                  
    Cached (Remote) = false                  
    Command         = echo 'building' > foo  
    Outputs         = foo                    
    Log File        = .titan/titan-build.log 
    Dependencies    =                        
    Dependendents   =                        

  $ ${TITAN} run build --dry=json --single-package
  {
    "tasks": [
      {
        "task": "build",
        "hash": "e1872eef9417d83e",
        "command": "echo 'building' \u003e foo",
        "outputs": [
          "foo"
        ],
        "excludedOutputs": null,
        "logFile": ".titan/titan-build.log",
        "dependencies": [],
        "dependents": []
      }
    ]
  }
