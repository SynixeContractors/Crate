#include "script_component.hpp"

if !(hasInterface) exitWith {};
if !(isMultiplayer) exitWith {};

if !(GVAR(enabled)) exitWith {};

["loadout", FUNC(loadout_onChange)] call CBA_fnc_addPlayerEventHandler;

if !(GVAR(shop_enabled)) exitWith {};

GVAR(shop_locked) = false;

call FUNC(shop_init);
