#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(GVAR(enabled)) exitWith {};

GVAR(objects_pfh) = [FUNC(objects_tick)] call CBA_fnc_addPerFrameHandler;
GVAR(groups_pfh) = [FUNC(groups_tick)] call CBA_fnc_addPerFrameHandler;
GVAR(units_pfh) = [FUNC(units_tick)] call CBA_fnc_addPerFrameHandler;
GVAR(markers_pfh) = [FUNC(markers_tick), 2] call CBA_fnc_addPerFrameHandler;

{
    _x setVariable [QGVAR(ignore), true];
} forEach allMissionObjects "All";

["grad_civs_lifecycle_civ_added", {
    params ["_civ"];
    _civ setVariable [QGVAR(ignore), true];
}] call CBA_fnc_addEventHandler;

["grad_civs_cars_car_added", {
    params ["_vehicle"];
    _vehicle setVariable [QGVAR(ignore), true];
}] call CBA_fnc_addEventHandler;

EXTCALL("campaigns:objects:load",[GVAR(key)]);
