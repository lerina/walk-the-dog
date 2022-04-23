#!/bin/sh

# USAGE:
# ./build.sh <target_name> <canvas_name_in_src_rust>
# ex:
# ./build.sh verify canvas002
#
TARGET=$1
CANVAS=$2

set -ex

wasm-pack build --target web

if [ -d "$TARGET" ]; then
    printf '%s\n' "Removing previous ($TARGET)"
    rm -rf "$TARGET"
fi
# mkdir $TARGET
mv pkg $TARGET

echo "<h3>$TARGET:</h3> <canvas id="$CANVAS" height="250" width="350"> </canvas> <script type="module"> import init, {} from './$TARGET/canvas.js'; async function run() { await init(); } run(); </script>" > $TARGET.html

printf '%s\n' "serving page at: http://127.0.0.1:8080"
#python3 -m http.server
firefox http://127.0.0.1:8080/$TARGET.html &
http -a 127.0.0.1 -p 8080
