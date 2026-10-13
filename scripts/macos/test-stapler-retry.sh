#!/usr/bin/env bash

set -euo pipefail
source "$(cd "$(dirname "$0")" && pwd)/common.sh"

run_case() (
  local name="$1" expected_status="$2" expected_calls="$3" expected_delays="$4" action="$5"
  shift 5
  local statuses=("$@")
  local calls=0 delays="" actual_status=0
  local artifact="/fixture/con Beta.app"

  xcrun() {
    [[ "$1" == "stapler" && "$2" == "$action" && "${!#}" == "$artifact" ]] \
      || fail "$name: arguments were changed"
    if [[ "$action" == "staple" ]]; then
      [[ "$#" == "4" && "$3" == "-v" ]] || fail "$name: staple flags were changed"
    else
      [[ "$#" == "3" ]] || fail "$name: validate flags were changed"
    fi
    calls=$((calls + 1))
    return "${statuses[$((calls - 1))]}"
  }
  sleep() {
    delays="${delays:+$delays }$1"
  }
  local args=("$action")
  if [[ "$action" == "staple" ]]; then
    args+=("-v")
  fi
  if run_stapler "${args[@]}" "$artifact"; then
    actual_status=0
  else
    actual_status=$?
  fi
  [[ "$actual_status" == "$expected_status" ]] || fail "$name: wrong final status $actual_status"
  [[ "$calls" == "$expected_calls" ]] || fail "$name: wrong attempt count $calls"
  [[ "$delays" == "$expected_delays" ]] || fail "$name: wrong backoff '$delays'"
  printf 'PASS: %s\n' "$name"
)

run_case 'first attempt succeeds' 0 1 '' staple 0
run_case 'ticket download recovers' 0 3 '10 20' staple 68 68 0
run_case 'validation recovers' 0 2 '10' validate 68 0
run_case 'service failure stays fatal after bounded retries' 68 4 '10 20 40' staple 68 68 68 68
for status in 64 65 66 73 77; do
  run_case "permanent failure $status is not retried" "$status" 1 '' staple "$status"
done
run_case 'permanent failure after retry stays fatal' 65 2 '10' staple 68 65
