Setup
  $ . ${TESTDIR}/../setup.sh
  $ . ${TESTDIR}/setup.sh $(pwd)

Check
  $ ${TITAN} run build --dry --single-package
  
  Tasks to Run
  build
    Task            = build                  
    Hash            = f46425039e0a4d15       
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
        "hash": "f46425039e0a4d15",
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
