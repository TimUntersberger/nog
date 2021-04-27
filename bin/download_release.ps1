param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $Release
)

$asset_url = "https://github.com/TimUntersberger/nog/releases/download/$Release/Nog.zip"
$asset_name = "__temp_release_asset"
$out_path = "$env:APPDATA/nog_lua"

invoke-webrequest $asset_url -outfile "$asset_name.zip"
expand-archive "./$asset_name.zip" "$asset_name"
move-item "$asset_name/Nog" $out_path

remove-item "./$asset_name.zip"
remove-item "./$asset_name"
