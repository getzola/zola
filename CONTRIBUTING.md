# Contributing
**As the documentation site is automatically built on commits to master, all development happens on
the `next` branch, unless it is fixing the current documentation.**

However, if you notice an error or typo in the documentation, feel free to directly submit a PR without opening an issue.

## Feature requests
If you want a small feature added or modified, please open an issue to discuss it before doing a PR.

Requested features will not be all added: an ever-increasing features set makes for a hard to use and explain softwares.
Having something simple and easy to use for 90% of the use cases is more interesting than covering 100% use cases after sacrificing simplicity.

If the feature is large, please open a PR with a [RFC](rfcs/000-template.md) on the `master` branch. If you're not
sure what qualifies as large, start with an issue. 


## Issues tagging

As the development happens on the `next` branch, issues are kept open until a release containing the fix is out.
During that time, issues already resolved will have a `done` tag.

If you want to work on an issue, please mention it in a comment to avoid potential duplication of work. If you have
any questions on how to approach it do not hesitate to ping me (@keats).
Easy issues are tagged with `help wanted` and/or `good first issue`

## Adding syntax highlighting languages, themes or aliases

Open an issue on the [Giallo repository](https://github.com/getzola/giallo).

## LLM usage

It's ok to use LLMs to review/find issues and help you understand the codebase if you're not familiar with it.
If you're using a LLM to write code, a human needs to review it, edit it, test it and stand by it.

If it's slop, you're getting banned.
If all the interactions feels like talking to a LLM, you're getting banned too.

LLM usage is not accepted for the documentation, no one wants to read LLM generated documentation. No one
wants to read LLM comments either so use your own words in issues/PR.
