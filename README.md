# Total War Patcher (TWPatcher)

**Redistributable Load Order Patcher** for all Total War games since Empire Total War. 

Basically, it generates a pack with custom patches for your specific load order, such as the classic **Skip Intro** and the revolutionary **make mods actually show text properly if the game is not in english** that you usually need the UPC for.

Supports only Steam-Installed games and Windows.

# Usage

To use it, open a CLI wherever you have the *twpatcher.exe* file (Shift+Right-Click, then hit *Open PowerShell Window Here*), then run it with the params you want. Here are some examples:
```bash

# For use with the vanilla launcher, for Attila, just to fix the "Missing text when the game is not in english" bug for the spanish language. 
./twpatcher.exe -g attila -l used_mods.txt -t sp

# For use with the vanilla launcher, for Three Kingdoms, just make all units bigger without altering their balance (like a 1.5X mod from the workshop). 
./twpatcher.exe -g three_kingdoms -l used_mods.txt -m "1.5"

# For use with Runcher, for Warhammer 3, skips update checks, skips intros, removes the trait limit,
# translates the mods it has translations for to spanish, and applies an unit multiplier of 1.5X. 
./twpatcher.exe -g warhammer_3 -l mod_list.txt -e -s -i -r -t sp -m "1.5"

```

# Current Features:

- **Enable Script Logging**: makes the game generate log files for scripts. Extremely useful to debug scripts, but makes the game slower. Supported only in: Warhammer 2, Warhammer 3, Troy, Pharaoh, Pharaoh Dynasties.

- **Skip Intro Videos**: skips all the intro videos, and unlike other mods that do this, it works with all languages. Supported in all games.

- **Remove Trait Limit**: removes the trait limit of characters in Warhammer 3. May cause your saves to be a bit bigger though.

- **Remove Siege Attacker**: removes the "Siege Attacker" attribute from everything but Warmachines in Warhammer 3. This should reduce a lot the instances of the AI attacking walled settlements without artillery or siege equipment, and overall make walled settlements... actually do something different than unwalled settlements.

- **Translation Fixer**: to be honest, I'm not sure how to call this one, but was one of the main reasons I started this. You tell it the language your game is, and it'll:
    - Scan your mods, and **fix all the missing text issues** that have plagued all games since Empire up to Thrones for ages. And yes, this kinda makes UPC obsolete.
    - Cleans your mods' texts, so a mod adding two units doesn't turn half your game into english. If they have text lines identical to the ones in the vanilla english loc files, they're replaced with their equivalent in your language.
    - **Automatically applies translations** from [here](https://github.com/Frodo45127/total_war_translation_hub), if any of the mods you use has one available for your language, avoiding all the problems translation packs have, like translations becoming outdated and no longer being usable without bugs, or the translation packs using up one of the packs you could use for other mods. Meaning this feature makes translation packs obsolete, though it needs translators to upload their translations to the [Translations Repo](https://github.com/Frodo45127/total_war_translation_hub) on Github.

- **Unit Multiplier**: multiplies all unit sizes by the value you provide. In case of single entities, it multiplies their health instead. It also takes care of multiplying certain parameters that scale with difficulty, like tower and magic damage, to try to not alter the balance you had in the game. Supported only in: Warhammer 3, Three Kingdoms.

# Redistribution

For modders which want to use this patcher as part of their custom mod managers or their custom launchers for their mods, you are free to redistribute this patcher with your mod/mod manager. Just remember to pass -s to the patcher so it doesn't download new updates automatically, because while I'll try to keep it backward-compatible, I don't guarantee an update may break something.

# Uninstall

To remove the program, simply delete the exe. To remove the generated patch, go to data, and delete the created Pack.

If you didn't specified a name for the Pack, the default names are (varies depending on the game): **zzzzzzzzzzzzzzzzzzzzrun_you_fool_thron.pack** and **!!!!!!!!!!!!!!!!!!!!!run_you_fool_thron.pack**. Don't ask why for the weird names. Took a while to figure out valid names for packs that skip intros properly.
