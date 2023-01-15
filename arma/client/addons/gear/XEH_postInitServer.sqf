#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(GVAR(enabled)) exitWith {};

// Delete bodies on disconnect
addMissionEventHandler ["HandleDisconnect", {
    params ["_unit", "_id", "_uid", "_name"];
    if !(isNil "GRAD_slingHelmet_fnc_weaponHolder") then {
        private _slungWH = [_unit] call GRAD_slingHelmet_fnc_weaponHolder;
        if (_slungWH != objNull) then {
            deleteVehicle _slungWH;
        };
    };
    [{
        deleteVehicle _this;
    }, _unit] call CBA_fnc_execNextFrame;
    false
}];

if !(GVAR(shop_enabled)) exitWith {};

publicVariable QGVAR(shop_boxes);

EXTFUNC("gear:shop:items");
