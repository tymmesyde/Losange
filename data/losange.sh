#! /bin/bash

# Use OpenGL GSK renderer for Nvidia cards
if ls /dev/nvidia0 &>/dev/null 2>&1; then
    export GSK_RENDERER=opengl
fi

exec /app/libexec/losange "$@"