Setup
  $ . ${TESTDIR}/../setup.sh
  $ . ${TESTDIR}/setup.sh $(pwd)

Check
  $ ${TITAN} run test --dry --single-package
  
  Tasks to Run
  build
    Task            = build                  
    Hash            = 6dab35f71988610e       
    Cached (Local)  = false                  
    Cached (Remote) = false                  
    Command         = echo 'building' > foo  
    Outputs         = foo                    
    Log File        = .titan/titan-build.log 
    Dependencies    =                        
    Dependendents   = test                   
  test
    Task            = test                                         
    Hash            = a4039f6c1959b8db                             
    Cached (Local)  = false                                        
    Cached (Remote) = false                                        
    Command         = [[ ( -f foo ) && $(cat foo) == 'building' ]] 
    Outputs         =                                              
    Log File        = .titan/titan-test.log                        
    Dependencies    = build                                        
    Dependendents   =                                              

  $ ${TITAN} run test --dry=json --single-package
  {
    "tasks": [
      {
        "task": "build",
        "hash": "6dab35f71988610e",
        "command": "echo 'building' \u003e foo",
        "outputs": [
          "foo"
        ],
        "excludedOutputs": null,
        "logFile": ".titan/titan-build.log",
        "dependencies": [],
        "dependents": [
          "test"
        ]
      },
      {
        "task": "test",
        "hash": "a4039f6c1959b8db",
        "command": "[[ ( -f foo ) \u0026\u0026 $(cat foo) == 'building' ]]",
        "outputs": null,
        "excludedOutputs": null,
        "logFile": ".titan/titan-test.log",
        "dependencies": [
          "build"
        ],
        "dependents": []
      }
    ]
  }
