$asset_url = "https://github.com/neovim/neovim/releases/download/v0.4.4/nvim-win64.zip"
$asset_name = "__temp_release_asset"
$out_path = "$env:APPDATA/nog_lua"

invoke-webrequest $asset_url -outfile "$asset_name.zip"
expand-archive "./$asset_name.zip" "$asset_name"
move-item "$asset_name/Neovim" $out_path

remove-item "./$asset_name.zip"
remove-item "./$asset_name"
