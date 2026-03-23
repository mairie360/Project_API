#!/bin/bash
set -e

# S'assurer qu'on est au bon endroit
cd /app

# Lancer cargo watch
# -x run : compile et lance l'exécutable
# Le cache sera conservé dans /app/target grâce au volume Docker
exec cargo watch -w src -i target -x run