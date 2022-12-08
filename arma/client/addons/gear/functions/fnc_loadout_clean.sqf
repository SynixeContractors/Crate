#include "script_component.hpp"

params ["_loadout"];

_loadout params ["_arma", "_extra"];

_arma = [_arma] call acre_api_fnc_filterUnitLoadout;

[_arma, _extra]
