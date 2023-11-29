#!/bin/bash
set -eu -o pipefail

git init law-html && cd law-html
git checkout -b main

make_index_html() {
    local html_content='<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <title>index.html</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" type="text/css" media="screen" href="/static/main.css">
    <script src="/static/main.js"></script>
</head>
<body>
    <h1>index.html</h1>
    <img src="/static/logo.png"/>
    <p id="js"></p>
</body>
</html>
'

    local path=$1
    local file_name=$2

    mkdir -p "$path"
    echo "$html_content" > "$path/$file_name"
}



make_index_html "./" "index.html"
make_index_html "./a" "index.html"
make_index_html "./a/b/d" "index.html"
make_index_html "./a/b" "c.html"
git add . && git commit -q -m c1


update_index_html() {
    local path=$1
    local file_name=$2
    local update_comment="<!-- Document updated -->"

    if [ -f "$path/$file_name" ]; then
        echo "$update_comment" >> "$path/$file_name"
    else
        echo "Error: File $path/$file_name does not exist."
    fi
}

update_index_html "./a/b" "c.html"
git add ./a/b/c.html && git commit -q -m c2