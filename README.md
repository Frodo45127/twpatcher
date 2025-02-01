# Total War Patcher (TWPatcher)

Load Order Patcher for all Total War games since Empire Total War. Basically, it generates a pack with patches for your specific load order, such as UPC-like custom patches, or Unit Multiplier patches.

Supports only Steam-Installed games and Windows.

# Usage

```bash

# Uses the mod_list.txt file for WH3, skips update checks, skips intros, removes the trait limit,
# translates the mods it has translations for to spanish, and applies an unit multiplier of 1.5X. 
./twpatcher.exe -g warhammer_3 -l mod_list.txt -e -s -i -r -t sp -m "1.5"

```

# Uninstall

To remove the program, simply delete the exe. To remove the generated patch, go to data, and delete the created Pack.

If you didn't specified a name for the Pack, the default names are (varies depending on the game): **zzzzzzzzzzzzzzzzzzzzrun_you_fool_thron.pack** and **!!!!!!!!!!!!!!!!!!!!!run_you_fool_thron.pack**. Don't ask why for the weird names.
