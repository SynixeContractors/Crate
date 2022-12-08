#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(GVAR(enabled)) exitWith {};

["loadout", FUNC(loadout_onChange)] call CBA_fnc_addPlayerEventHandler;
