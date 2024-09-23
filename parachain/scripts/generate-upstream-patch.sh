#!/bin/bash

set -eo pipefail

ROOTDIR=$(git rev-parse --show-toplevel)
UPSTREAM_URL_PREFIX="https://github.com/integritee-network"

function usage() {
	echo "Usage:"
	echo "For worker : $0 [worker-new-commit]"
}

function add_upstream_url() {
	local target=$1
	local upstream_url="$UPSTREAM_URL_PREFIX/$target"
	if [ "$(git remote get-url "upstream_$target" 2>/dev/null)" != "$upstream_url" ]; then
		git remote add "upstream_$target" "$upstream_url"
	fi
}

# This function generates a patch for the diffs between commit-A and commit-B
# of the upstream repo, where
# 	- commit-A: the commit recorded in tee-worker/upstream_commit
# 	- commit-B: the HEAD of the given commit
# The patch will be generated under tee-worker/
function generate_worker_patch() {
	local dest_dir="$ROOTDIR/tee-worker"
	local upstream_url="$UPSTREAM_URL_PREFIX/worker"

	cd "$dest_dir"

	if [ -f upstream_commit ]; then
		OLD_COMMIT=$(head -1 upstream_commit)
	else
		echo "Can't find upstream_commit file in $dest_dir, quit"
		exit 1
	fi

	echo "fetching upstream_worker ..."
	git fetch -q "upstream_worker"

	local tmp_dir
	tmp_dir=$(mktemp -d)
	cd "$tmp_dir"
	echo "cloning $upstream_url to $tmp_dir"
	git clone -q "$upstream_url"
	cd worker && git checkout "$1"
	echo "generating patch ..."
	git diff "$OLD_COMMIT" HEAD > "$dest_dir/upstream.patch"
	git rev-parse --short HEAD > "$dest_dir/upstream_commit"
	rm -rf "$tmp_dir"
	echo "done"
}

function apply_worker_tips() {
	echo "======================================================================="
	echo "upstream_commit(s) are updated."
	echo "upstream.patch(s) are generated."
	echo "To apply it, RUN FROM $ROOTDIR:"
	echo "  git am -3 --exclude=tee-worker/Cargo.lock --exclude=tee-worker/enclave-runtime/Cargo.lock --directory=tee-worker < tee-worker/upstream.patch"

	echo ""
	echo "after that, please:"
	echo "- pay special attention: "
	echo "  * ALL changes/conflicts from tee-worker/upstream.patch patch should ONLY apply into:"
	echo "    - tee-worker"

	echo "- resolve any conflicts"
	echo "- optionally update Cargo.lock file"
	echo "- apply any changes of workflows/build_and_test.yml to $ROOTDIR/.github/workflows/ci.yml"
}

NEW_COMMIT=${1:-master}

add_upstream_url "worker"
generate_worker_patch "$NEW_COMMIT"
apply_worker_tips
