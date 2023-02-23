#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(GVAR(enabled)) exitWith {};

GVAR(objects_pfh) = [FUNC(objects_tick), 0.1] call CBA_fnc_addPerFrameHandler;
GVAR(groups_pfh) = [FUNC(groups_tick), 0.1] call CBA_fnc_addPerFrameHandler;
GVAR(units_pfh) = [FUNC(units_tick), 0.1] call CBA_fnc_addPerFrameHandler;
GVAR(markers_pfh) = [FUNC(markers_tick), 2] call CBA_fnc_addPerFrameHandler;

EXTFUNC("campaigns:objects:load");
