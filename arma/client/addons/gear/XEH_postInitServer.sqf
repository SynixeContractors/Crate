#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(GVAR(enabled)) exitWith {};

// Delete bodies on disconnect
addMissionEventHandler ["HandleDisconnect", {
    params ["_unit", "_id", "_uid", "_name"];
    [{
        deleteVehicle _this;
    }, _unit] call CBA_fnc_execNextFrame;
    false
}];

if !(GVAR(shop_enabled)) exitWith {};

publicVariable QGVAR(shop_boxes);

EXTFUNC("gear:shop:items");
