#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"

// Don't erase existing, in the freestyle they are added dynamically
if (isNil QGVAR(shop_boxes)) then {
    GVAR(shop_boxes) = [];
};

[
    QGVAR(enabled),
    "CHECKBOX",
    ["Enabled", "Enable Persistent Gear"],
    "Crate - Persistent Gear",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;

[
    QGVAR(readOnly),
    "CHECKBOX",
    ["Read Only", "Load gear when the mission starts, but do not save changes"],
    "Crate - Persistent Gear",
    false,
    true,
    {},
    true
] call CBA_fnc_addSetting;

[
    QGVAR(shop_enabled),
    "CHECKBOX",
    ["Enabled", "Are shops usable?"],
    ["Crate - Persistent Gear", "Shops"],
    true,
    true,
    {},
    true
] call CBA_fnc_addSetting;

ADDON = true;
