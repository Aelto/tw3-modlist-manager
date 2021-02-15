# Script merging
## What is script merging
Script merging is the process of combining two modified files A and B into an
original version of the same file. files A and B both contain unique changes to
the original file and should both be edited version of the original file.

For example, with an original file with the following code:
```js
function foo() {
  var a: bool;

  a = this.should_a();
  
  if (a) {
    // ...
  }
}
```

and a version A with the following code:
```js
function foo() {
  var a: bool;
  var b: bool;

  a = this.should_a();
  b = this.should_b();
  
  if (a && b) {
    // ...
  }
}
```

and a version B with the following code:
```js
function foo() {
  var a: bool;
  var c: bool;

  a = this.should_a();
  
  if (a) {
    // ...
  }
  else {
    c = false;
    // ...
  }
}
```

the result should look like this:
```js
function foo() {
  var a: bool;
  var b: bool;
  var c: bool;

  a = this.should_a();
  b = this.should_b();
  
  if (a && b) {
    // ...
  }
  else {
    c = false;
    // ...
  }
}
```

## Implementation
There is a known implementation of script merging that is commonly used by
people and that uses a `mergeinventory.xml` file store information about the
current state of the merged files.

It is something this implementation will not do, and instead will follow the
same principle famous unix tools like nginx do where everything is a file.
Each modlist will have a new folder `merges` where the merger will copy the
architecture of the merged files and will create symlinks to the files the user
accepted to merge.

If we have the current modlist:
```
mods/
  modA/
    actor.ws
    player.ws

  modB/
    actor.ws
    player.ws
```

and the user merged `actor.ws` but not `player.ws` then the `merges` folder will
look like this:
```
merges/
  modA/
    actor.ws (symlink)
  
  modB/
    actor.ws (symlink)
```

the resulting merge will be created in `merges/mod_000000_mergedfiles`. And when
the modlist is installed a symlink `mods/mod_000000_mergedfiles` pointing to `merges/mod_000000_mergedfiles` will be created if it doesn't already exist.

The tool will start by filling the `mod_000000_mergedfiles` with the first version of every file that can be found
in the `/merges` folder. And then it will go from top to bottom and for each
file it sees it will `merge(original, A, B)` with the vanilla file as the original, the file in the `mod_000000_mergedfiles` folder as the version A
and the current file for the version B. If it is a success, the output will
overwrite the current file in the `mod_000000_mergedfiles` folder. If there is
a conflict it will prompt the user about the conflict and will wait for him to
pick over Original, A, or B and the result will overwrite the current file in 
the `mod_000000_mergedfiles` folder too.

### Merge imports
This implementation allows us to implement merge imports. If an imported modlist
has a `mod_000000_mergedfiles` folder in `/merges` then a symlink with the
following naming convention `mod_000<import-order>_<modlist-name>_mergedfiles` 
will be created and will point to the `mod_000000_mergedfiles`

> NOTE: the three `0` compared to the six `0` in the current modlist merged directory allows the current mergedfiles to be loaded first, and then come 
> imported modlists based on their import order. So the first imported will be
> create a symlink named `mod_000001_shared.visuals_mergedfiles`.
