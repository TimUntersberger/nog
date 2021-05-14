# Nog

![preview](https://user-images.githubusercontent.com/32014449/107612664-0490ac00-6c47-11eb-9620-e754aa38b5b0.png)

## Documentation

https://timuntersberger.github.io/nog

## Download

### Windows

```powershell
(iwr "https://raw.githubusercontent.com/TimUntersberger/nog/master/bin/download_release.ps1").Content > download.ps1; ./download.ps1 master-release; rm download.ps1
```

## Known Problems

### Window gets managed on wrong monitor

If you are using something like PowerLauncher for launching applications you might encounter this problem with `mutli_monitor` enabled.

The problem is that the focus returns to the previous window after PowerLauncher closes, before spawning the new window.

1. PowerLauncher opens
2. You tell it to launch notepad for example
3. PowerLauncher closes -> focus returns to previous application
4. notepad launches

If the previous application mentioned in step 3 is managed by nog, the workspace will change to its grid. The only way to fix this (at least that I know of) is if we implement our own application launcher that is connected with nog. 

## Contributions

* Thank you [@ramirezmike](https://github.com/ramirezmike) for designing and implementing the graph based tile organizer

## Development

Nog requires `nightly` rust.

### Make Release

```
./bin/make_release.ps1 <VERSION>
```

### Serve documentation

This requires you to have [mdbook](https://github.com/rust-lang/mdBook) installed.

The command will serve the book at `https://localhost:3000` and automatically rebuild whenever you change the source.

```
mdbook serve ./book
```

### Build documentation

This requires you to have [mdbook](https://github.com/rust-lang/mdBook) installed.

The command will build the book directory and output the generated files into the docs directory.

```
mdbook build ./book
```

### Updating .ns Config Files
We recently changed the config scripting language to use Lua. If you need help converting your config to the new format, consult the config guide [here]( https://github.com/TimUntersberger/nog/blob/master/config.md) or feel free to post on the [documentation feedback issue](https://github.com/TimUntersberger/nog/issues/106).
