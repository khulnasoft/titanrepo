Setup
  $ . ${TESTDIR}/../setup.sh
  $ . ${TESTDIR}/setup.sh $(pwd)

Check
  $ ${TITAN} run build --dry --single-package
  
  Tasks to Run
  build
    Task            = build                  
    Hash            = 31cff621047ffed4       
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
        "hash": "31cff621047ffed4",
        "command": "echo 'building'",
        "outputs": null,
        "excludedOutputs": null,
        "logFile": ".titan/titan-build.log",
        "dependencies": [],
        "dependents": []
      }
    ]
  }
