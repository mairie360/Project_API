#!/bin/bash
set -e

# CORRECTION : Utiliser le même dossier que le WORKDIR du Dockerfile
cd /usr/src/calendar_api

# Lancer cargo watch
exec cargo watch --poll -w src -i target -x run
