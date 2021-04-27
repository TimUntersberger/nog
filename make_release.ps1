param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $Version
)

$root_dir = "Nog"

$env:NOG_VERSION=$Version

cargo build -p twm --release

if (!$?) {
  echo "Build was not successful. Aborting."
  return
}

./rcedit.exe ./$root_dir/bin/nog.exe --set-icon ./assets/logo.ico

if (test-path ./$root_dir) {
  remove-item -Path ./$root_dir -Recurse
}

new-item -path . -name $root_dir -itemtype "Directory"
new-item -path ./$root_dir -name "runtime" -itemtype "Directory"
new-item -path ./$root_dir -name "bin" -itemtype "Directory"

copy-item ./twm/runtime/* ./$root_dir/runtime -Recurse
copy-item ./target/release/twm.exe ./$root_dir/bin/nog.exe
