#include "script_component.hpp"

if !(EGVAR(gear_main,enabled)) exitWith {};
if (EGVAR(gear_main,read_only)) exitWith {};

[QEGVAR(gear_shop,loaderRegister), QUOTE(ADDON)] call CBA_fnc_localEvent;

GVAR(tracking) = false;

GVAR(shop_loadout_before_open) = [player] call CBA_fnc_getLoadout;

[QGVAR(store), [player, BLANK_LOADOUT]] call CBA_fnc_serverEvent;
