# witcher-3-modlist-manager
Small utility to manage different mod lists for the game The Witcher 3 

# Installing

- download the binary from the [releases](https://github.com/Aelto/tw3-modlist-manager/releases)
- create a directory in your witcher install `The Witcher 3` called `modlists`, so that you have `The Witcher 3/modlists`

# Using it
On first launch the option to `initialize` is available. This option moves the necessary vanilla files into a modlist called `vanilla`, so the files will be moved into `The Witcher 3/modlists/vanilla` (in case you want to revert it). Of course, if your current install was a modded game, your vanilla modlist won't be exactly vanilla but your current install.

> I would advise to create yourself a real vanilla install. You will have trouble swapping between modlists because your vanilla modlist won't be a real vanilla install and som DLCs or content scripts will be modified.
>
> you will see later, but the vanilla modlist is important as it's a requirement for all other modlists. It contains the base game DLCs and scripts

## Creating a modlist
Once the you've initialized the modlist manager, you can now create a new modlist with the text input and the `new` button. A modlist is just a directory with 6 folders, if any of these six folders is missing the manager won't consider the modlist as valid and won't display it in the interface.

If you want to add mods to a modlist, do exactly as you would do with the game before. Drop the mods in the `mods` directory of the modlist, DLCs in the `dlcs` directory, changes in the content in the `content` directory, changes in the vanilla bundles go in the `bundles` directory,  new menus in the `menus` directory and changes in the keybinds and/or saves in the `saves` directory.
You can refer to the vanilla modlist to understand how to place the files and folders.

## Importing a modlist
You've created yourself a new modlist, but that's not enough. Imagine you added a new mod in your modlist and wanted to install it. The game won't launch because it's missing all the necessary vanilla files such as the content directory, and the sixteen or so base DLCs.

For this reason modlist imports were created. Your modlist can import the vanilla modlist to inherit all of its content, the vanilla files. **this is why you must ALWAYS import the vanilla modlist**.

modlist imports occur from top to bottom. And if an imported modlist tries to import a file that already exists in your current modlist, it will ignore it and won't import anything. So you can edit the load order of the imported modlists with the little up/down arrows. **the vanilla modlist should always be last import to avoid erasing your modified files**.

After your changes to the import list, you can choose to unload or load all imports. When you load imports it creates a series of symlinks linking to the files from the different modlists you imported. Be careful, as these files are not copies but instead shortcuts to the real files in the other modlists. If you edit them, the original files will be edited too and it may break the imported modlist.

So before doing any changes to your modlist, i would advise to unload the imports. And to load the import only when you're sure you won't touch anything sensitive mod file.

## Installing a modlist
Ok so now you have a new modlist and its imports are all set up. You can install it by going to the home page (by clicking the modlist manager title) and you can click the `install` button right next to its name and the modlist will be installed. The process is almost instant so you may not see it, but it was installed. 

You can now launch the game and confirm the modlist you just installed is valid.

## Merging a modlist
You're maybe wondering how it will work with your merged files. The modlist manager supports mergeinventory swapping too. The mergeinventory is what scriptmerger uses to store which mod you merged and which one you installed.

If you place a `MergeInventory.xml` file in the root of your modlist (example: `The Witcher 3/modlists/ghostmode/MergeInventory.xml`) when you will install the modlist, if you have scriptmerger installed in the directory `The Witcher 3/scriptmerger` and there is no mergeinventory in the scriptmerger install directory, the modlist manager will swap the mergeinventory to the one used by your modlist.

So imagine i want to merge all my mods used by the `ghostmode` modlist, which imports the two modlists `vanilla` and `random-encounters-reworked`:
- I go load all imports to inherit from the vanila and RER files
- Then i install the modlist, which places the mergeinventory in the scriptmerger install and also swap the `The Witcher 3/mods` directory (used by scriptmerger)
- I confirm i didn't inherit from the mergedfiles from another modlist i imported by going in `The Witcher 3/mods`, and if i did, i delete the directory (no risk here, you're just deleting the shortcut and not the actual merge).
- I open script merger and proceed with the merge.