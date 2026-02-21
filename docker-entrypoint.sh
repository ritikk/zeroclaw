#!/bin/sh
set -e

zeroclaw daemon &
exec zeroclaw gateway
