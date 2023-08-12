#!/bin/sh

function upload_to_package_registry() {
	upload_file=$1
	upload_target=$2

	curl --header "JOB-TOKEN: ${CI_JOB_TOKEN}" --upload-file $upload_file ${PACKAGE_REGISTRY_URL}/$upload_target
}

upload_to_package_registry "bin/$FINAL_BIN_NAME" "$FINAL_BIN_NAME"

for file in $(ls bin/tab_completions/); do
	file_path="bin/tab_completions/$file"
	upload_to_package_registry $file_path "tab_completion_$file"
done
