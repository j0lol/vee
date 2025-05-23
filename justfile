# This justfile is for my convenience only.
# Please don't use this unless you need to.

# Yeah, idk.
xcode:
    cargo build --bin lightweight_viewer
    open /private/var/folders/sl/mys5xs454210crk192gs9sk00000gn/T/BED4E4F3-9BCC-436E-992A-9352729F3DFA/lightweight_viewer.xcworkspace

profile:
    just profile-instrument

# Requires samply.
profile-samply:
    cargo build --bin lightweight_viewer --profile profiling
    BROWSER=/Applications/Firefox\ Nightly.app/Contents/MacOS/firefox samply record ./target/profiling/lightweight_viewer
profile-samply-debug:
    cargo build --bin lightweight_viewer
    BROWSER=/Applications/Firefox\ Nightly.app/Contents/MacOS/firefox samply record ./target/debug/lightweight_viewer

# Requires Xcode (Instruments).
profile-instrument:
    cargo build --bin lightweight_viewer --profile profiling
    rm -rf /tmp/tracefile.trace
    xcrun xctrace record \
    --template "Time Profiler" \
    --output /tmp/tracefile.trace \
    --launch -- ./target/profiling/lightweight_viewer
    open /tmp/tracefile.trace
