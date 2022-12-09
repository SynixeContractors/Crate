#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[
    QGVAR(roles_enabled),
    "CHECKBOX",
    ["Load Roles", "Load roles from the Discord server"],
    "Crate - Discord",
    true,
    true,
    {},
    true
] call CBA_fnc_addSetting;
