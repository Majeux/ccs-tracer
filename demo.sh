#!/usr/bin/env bash

# set -Eeuo pipefail

readonly examples=(
	"actions.ccs"
	"restrict.ccs"
	"relabel.ccs"
	"restrict_and_relabel.ccs"
	"restrict_and_relabel2.ccs"
	"choice.ccs"
	"parallel.ccs"
	"composition.ccs"
	"recursion.ccs"
	"big.ccs"
)

run_example() {
	path=$1
	content=$(cat $path)

	clear
	printf "example: $(basename $path .css): \n"
	printf -- "------------------------\n"
	printf "$content\n"
	printf -- "------------------------\n\n"

	./target/debug/ccs-tracer "${path}"
	read -p "" </dev/tty
}

for example in "${examples[@]}"; do
	path=$(echo "example_input/${example}")
	run_example $path
done
