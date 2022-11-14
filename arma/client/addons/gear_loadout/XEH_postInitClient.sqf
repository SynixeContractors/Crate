#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(EGVAR(gear_main,enabled)) exitWith {};

["loadout", FUNC(handle_onChange)] call CBA_fnc_addPlayerEventHandler;
