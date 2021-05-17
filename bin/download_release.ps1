param(
  [Parameter(Mandatory=$True, Position=0)]
  [string]
  $Release
)

$asset_url = "https://github.com/TimUntersberger/nog/releases/download/$Release/Nog.zip"
$asset_name = "__temp_release_asset"
$out_path = "$env:APPDATA/nog"
$config_was_saved = $false

invoke-webrequest $asset_url -outfile "$asset_name.zip"
expand-archive "./$asset_name.zip" "$asset_name"

if (test-path $out_path) {
  echo "Saving config"
  move-item $out_path/config $env:APPDATA/nog_tmp_config
  $config_was_saved = $true
  echo "Removing previous nog version"
  remove-item -path $out_path -Recurse
}

move-item "$asset_name/Nog" $out_path

if ($config_was_saved) {
  move-item $env:APPDATA/nog_tmp_config $out_path/config 
}

remove-item "./$asset_name.zip"
remove-item "./$asset_name"
