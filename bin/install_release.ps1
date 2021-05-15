param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $file
)

$asset_name = "__temp_release_asset"
$out_path = "$env:APPDATA/nog"

expand-archive $file $asset_name

if (test-path $out_path) {
  echo "Removing previous nog version"
  remove-item -path $out_path -Recurse
}

move-item "$asset_name/Nog" $out_path

remove-item $file
remove-item "./$asset_name"
