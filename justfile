# This justfile is for my convenience only.
# Please don't use this unless you need to.
#
# Xcode GPU DEBUGGING:
# How to get your own .xcworkspace (the stupid way)
# Select Menu Item: Xcode -> Debug executable -> select lightweight_viewer
# Select Menu Item: File -> Workspace settings
# In popup: Derived Data -> Workspace-relative Location
# Press the little (->) button by the path to open it in Finder
# Copy the .xcworkspace file to somewhere safe

xcode:
    cargo build --bin lightweight_viewer
    open ~/Projects/lightweight_viewer.xcworkspace

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
