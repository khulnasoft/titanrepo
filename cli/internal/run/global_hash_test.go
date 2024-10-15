package run

import (
	"reflect"
	"testing"
)

func Test_getHashableTitanEnvVarsFromOs(t *testing.T) {
	env := []string{
		"SOME_ENV_VAR=excluded",
		"SOME_OTHER_ENV_VAR=excluded",
		"FIRST_THASH_ENV_VAR=first",
		"TITAN_TOKEN=never",
		"SOME_OTHER_THASH_ENV_VAR=second",
		"TITAN_TEAM=never",
	}
	gotNames, gotPairs := getHashableTitanEnvVarsFromOs(env)
	wantNames := []string{"FIRST_THASH_ENV_VAR", "SOME_OTHER_THASH_ENV_VAR"}
	wantPairs := []string{"FIRST_THASH_ENV_VAR=first", "SOME_OTHER_THASH_ENV_VAR=second"}
	if !reflect.DeepEqual(wantNames, gotNames) {
		t.Errorf("getHashableTitanEnvVarsFromOs() env names got = %v, want %v", gotNames, wantNames)
	}
	if !reflect.DeepEqual(wantPairs, gotPairs) {
		t.Errorf("getHashableTitanEnvVarsFromOs() env pairs got = %v, want %v", gotPairs, wantPairs)
	}
}
