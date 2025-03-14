# find-your-godot `fyg`

> _Stop waiting. Find the right Godot for your project today!_

`fyg` is a command line version manager for the [Godot game engine]. It can install engine versions
and open your projects in the right engine given a `fyg.toml`.

[Godot game engine]: https://godotengine.org/

## Usage
```
$ fyg --help
A version manager for the Godot game engine.

Usage: fyg [COMMAND]

Commands:
  list       List Godot engine versions. Shows installed versions by default
  install    Install the given Godot engine version
  uninstall  Uninstall the given Godot engine version
  launch     Launch the given Godot engine version
  edit       Edit a Godot project with its associated Godot engine
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Managing Godot Versions
### Install
You can `list` versions of Godot available on GitHub:
```
$ fyg list -a
4.0.3
4.0.2
4.0.1
3.5.2
4.0
3.5.1
3.5
# ... the list continues
```

And `install` them:
```
$ fyg install 4.0.3
```

### Uninstall
You can `list` installed versions of Godot:
```
$ fyg list
4.0.3
3.5.2
```

And `uninstall` them:
```
$ fyg uninstall 4.0.3
```

## Working with Projects
You can associate a Godot project with a particular engine version by placing a `fyg.toml` file alongside its `project.godot` file.

The contents are simply:
```toml
version = "4.0.3"
```

Now you can `edit` your project with the associated version:
```sh
$ cd path/to/project
$ fyg edit
```
And Godot should launch with your project open!

You can also tell `fyg` where to find a `project.godot` if it's not in the same directory as the `fyg.toml` by adding:
```toml
root = "path/to/project"
```
to the project's `fyg.toml` file. The path is relative to the `fyg.toml`'s parent directory.

## Managing Download Cache
`fyg` caches downloads in a separate directory from where it installs engine files. You can manage the cache with the `cache` command.

By default it `show`s files in the cache:
```
$ fyg cache
4.0 (51.58 MB): C:\Users\MyUser\AppData\Local\find-your-godot\cache\engines\4.0-stable\Godot_v4.0-stable_win64.exe.zip
4.0.1 (52.00 MB): C:\Users\MyUser\AppData\Local\find-your-godot\cache\engines\4.0.1-stable\Godot_v4.0.1-stable_win64.exe.zip
Total: 103.59 MB
```

And you can remove engine cache files with the `rm` subcommand:
```
$ fyg cache rm 4.0.1
Removing C:\Users\MyUser\AppData\Local\find-your-godot\cache\engines\4.0.1-stable
$ fyg cache
4.0 (51.58 MB): C:\Users\MyUser\AppData\Local\find-your-godot\cache\engines\4.0-stable\Godot_v4.0-stable_win64.exe.zip
Total: 51.58 MB
```
