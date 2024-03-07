#include "script_component.hpp"

ADDON = false;

#include "XEH_PREP.hpp"

[
    QGVAR(destroyChance),
    "SLIDER",
    ["Item Destroy Chance", "Destroy chance of each item when placing a unit into a bodybag."],
    ["Synixe - Equipment", "Bodybag"],
    [0, 100, DESTROY_CHANCE_DEFAULT, 0],
    true
] call CBA_fnc_addSetting;


ADDON = true;
