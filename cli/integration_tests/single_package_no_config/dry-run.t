Setup
  $ . ${TESTDIR}/../setup.sh
  $ . ${TESTDIR}/setup.sh $(pwd)

Check
  $ ${TITAN} run build --dry --single-package
  
  Tasks to Run
  build
    Task            = build                  
    Hash            = 983caf383ff15c7a       
    Cached (Local)  = false                  
    Cached (Remote) = false                  
    Command         = echo 'building'        
    Outputs         =                        
    Log File        = .titan/titan-build.log 
    Dependencies    =                        
    Dependendents   =                        

  $ ${TITAN} run build --dry=json --single-package
  {
    "tasks": [
      {
        "task": "build",
        "hash": "983caf383ff15c7a",
        "command": "echo 'building'",
        "outputs": null,
        "excludedOutputs": null,
        "logFile": ".titan/titan-build.log",
        "dependencies": [],
        "dependents": []
      }
    ]
  }
