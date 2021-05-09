param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $Release
)

$asset_url = "https://github.com/TimUntersberger/nog/releases/download/$Release/Nog.zip"
$asset_name = "__temp_release_asset"
$out_path = "$env:APPDATA/nog"

invoke-webrequest $asset_url -outfile "$asset_name.zip"
expand-archive "./$asset_name.zip" "$asset_name"

if (test-path $out_path) {
  echo "Removing previous nog version"
  remove-item -path $out_path -Recurse
}

move-item "$asset_name/Nog" $out_path

remove-item "./$asset_name.zip"
remove-item "./$asset_name"
