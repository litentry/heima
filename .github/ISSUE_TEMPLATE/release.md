---
name: Release issue template
about: Tracking issue for new releases
title: Litentry-parachain {{ env.VERSION }} Release checklist
---
## Release Checklist

Here is a release checklist for Litentry-parachain {{ env.VERSION }}.

The current release candidate can be checked out with `git checkout release-{{ env.VERSION }}`

### before forking to a release branch

These checks should be performed prior to forking to a release branch, that is, on `dev` branch.

- [ ] Verify **`spec_version`** has been bumped since the last release in case of runtime change.
- [ ] Verify previously **`completed migrations`** are removed (or at least guarded with if-check if they shouldn't be removed for some reason)
- [ ] Verify pallet and extrinsic ordering has stayed the same. Bump **`transaction_version`** if not.
- [ ] Verify new extrinsics have been correctly whitelisted/blacklisted for **`baseCallFilter`**.
- [ ] Verify **`benchmarks`** have been updated for any modified runtime logic.

### after forking to a release branch

These checks should be performed after forking to a release branch, that is, on the `release-{{ env.VERSION }}` branch.

- [ ] Verify **`new migrations`** complete successfully, ideally tested with both unittests and `try-runtime` on a current state.
- [ ] Verify **`all tests`** pass for release codebase, including cargo tests and ts tests.
- [ ] Verify the new release (including runtime upgrade) can be run on **`staging`** env.
- [ ] Verify the new release (including runtime upgrade) can be run on **`public network`** env, like westend/rococo, if applicable.

### publish the release

These are the general checks for the releasing process.

- [ ] Verify a **`tag`** has been correctly acquired on `release-{{ env.VERSION }}` branch.
- [ ] Verify **`release-draft`** has been created by [create-release-draft.yml](https://github.com/litentry/heima/blob/dev/.github/workflows/create-release-draft.yml) and is ready to publish.
