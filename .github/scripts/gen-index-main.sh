#!/bin/bash
# Generate root redirect for main branch documentation

cat > index.html <<'HTML'
<!doctype html>
<meta charset="utf-8">
<meta http-equiv="refresh" content="0; url=./main/en/">
<link rel="canonical" href="./main/en/">
<title>ZeroClaw Docs</title>
HTML
