#include "script_component.hpp"

params ["_extendedLoadout"];

_extendedLoadout params ["_loadout", "_extra"];

_loadout = [_loadout] call acre_api_fnc_filterUnitLoadout;

[_loadout, _extra]
