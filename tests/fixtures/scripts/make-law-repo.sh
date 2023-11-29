#!/bin/bash
set -eu -o pipefail


git init law && cd law
git checkout -b main

mkdir -p targets

create_repositories_json() {
    local json_content='
{
  "repositories": {
    "test/law-html": {
      "custom": {
        "type": "html",
        "serve": "historical",
        "location_regex": "/",
        "routes": [".*"]
      }
    }
  }
}'
    echo "$json_content" > "targets/repositories.json"
}
create_repositories_json

git add targets/repositories.json && git commit -q -m c1
