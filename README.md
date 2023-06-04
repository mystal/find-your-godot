# find-your-godot (fyg)

> _Stop waiting. Find the right Godot for your project today!_

`fyg` is a command line version manager for the [Godot game engine]. It can install engine versions
and open your projects in the right engine given a `godot_version.toml`.

[Godot game engine]: https://godotengine.org/

## Usage

```
$ fyg --help
A version manager for the Godot game engine.

Usage: fyg.exe [COMMAND]

Commands:
  list       List Godot engine versions. Shows installed versions by default
  install    Install the given Godot engine version
  uninstall  Uninstall the given Godot engine version
  launch     Launch the given Godot engine version
  open       Open the Godot project in the current directory in its associated Godot engine
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
You can associate a Godot project with a particular engine version by placing a `godot_version.toml` file alongside its `project.godot` file.

The contents are simply:
```toml
version = "4.0.3"
```

Now you can `open` your project with the associated version:
```
$ cd path/to/project
$ fyg open
```
And Godot should launch with your project open!
