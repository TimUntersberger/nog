# Installation

Nog has a very useful powershell script which downloads a given release.

It basically just downloads the `Nog.zip` file from the assets, unzips it and moves it to the correct location.

### Usage

```powershell
./bin/download_release.ps1 <release_name>
```

So if you want to download the latest master release you would call the script like this:

```powershell
./bin/download_release.ps1 master-release
```

It is also possible to run the script without having to clone the repo with this simple one-liner

```powershell
(iwr "https://raw.githubusercontent.com/TimUntersberger/nog/master/bin/download_release.ps1").Content > download.ps1; ./download.ps1 master-release; rm download.ps1
```

The snippet those the following steps:

1. Download the `download_release.ps1` script locally as `download.ps1`
2. Executes the script with `master-release`
3. Removes the `download.ps1` script
