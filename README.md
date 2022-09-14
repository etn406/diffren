# diffren

Tool to rename lots of files and folders using a text editor with a "diff" view to compare currents and targets paths. For now it only works with VSCode but I plan to add the support of others/custom text editors.

## Configuration


### Set the text editor to use

```sh
$ diffren use-editor <TEXT_EDITOR>
```

-  `<TEXT_EDITOR>` Possible values: `vscode`, `vscodium`, `custom`.

#### Example

```sh
$ diffren use-editor vscodium
The editor to use is now: VSCodium
```


### Set the custom editor command

```sh
$ diffren set-custom-editor <COMMAND>
```

-  `<COMMAND>` The command to start the custom editor, with `{target}` that'll be replaced by the path to the file containing the target names to rename files to, and optionnally `{current}` that contains the current names to rename files from.

#### Example

```sh
$ diffren set-custom-editor "vscode --wait --diff {current} {target}"
The custom editor launch command has been set with:
  > vscode --wait --diff {current} {target}
```


### Read the current configuration

```sh
$ diffren get-config
```


### Rename

```
diffren run [PATHS]...
```

- `[PATHS]...`: Path(s) of the files to list. Unix shell style patterns are supported. Defaults to `*`.

#### Example

```
$ ls Chill\ Bump*

Chill Bump - Back to the Grain:
01 - Chill Bump - Matter of Choice (Intro).opus 04 - Chill Bump - Leakin'.opus
02 - Chill Bump - No Pressure.opus              05 - Chill Bump - Occupy (99%).opus
03 - Chill Bump - Watch Me Score Points.opus    06 - Chill Bump - It's Alive !.opus

Chill Bump - Starting From Scratch:
01 - Chill Bump - Lost in the Sound.opus             04 - Chill Bump - Water Boycotter.opus
02 - Chill Bump - My Mother Is a Pornstar.opus       05 - Chill Bump - Snip Snip.opus
03 - Chill Bump - The Smell of Beer (interlude).opus

$ diffren run Chill\ Bump*/*
```

It opens a VSCode window with the current file names on the left, and the target file names on the right:

![vscode-capture-diff](https://user-images.githubusercontent.com/1438257/190187454-66f768bc-25f3-4839-97ca-af60761aeb99.png)

You edit the target file on the right to plan the renamings:

![Capture d’écran 2022-09-14 à 16 50 20](https://user-images.githubusercontent.com/1438257/190188316-8c1795ed-32f3-438a-9614-4439b57a51c7.png)

The save and close the diff view ; _diffren_ is still running in the terminal and now asks you to confirm the changes :

![Capture d’écran 2022-09-14 à 16 55 21](https://user-images.githubusercontent.com/1438257/190189643-a3ace88b-0936-4964-981a-5b94acf7d01c.png)

_Confirm_ will apply the renamings and shows a recap:

![Capture d’écran 2022-09-14 à 16 56 46](https://user-images.githubusercontent.com/1438257/190190187-2bfaa922-5b3f-406f-aabd-16c66b99c8c2.png)

