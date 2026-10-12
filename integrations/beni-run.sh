#!/usr/bin/env bash
# Keep argument boundaries and the original command's exit status; never eval command strings.
exec "${BENI_CLI:-beni-buddy-cli}" run -- "$@"
