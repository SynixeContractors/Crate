#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

FUNC(fuelType) = {
    param ["_vehicle"];
    private _config = getText (configOf _vehicle >> "crate_fuel_type");
    if (_config == "") then {
        _config = switch (true) do {
            case (_vehicle isKindOf "Helicopter"): {
                "avgas"
            };
            case (_vehicle isKindOf "Plane"): {
                "jeta1"
            };
            default {
                "regular"
            };
        };
    };
    _config
};

[QGVAR(started), {
    if (EGVAR(gear,readOnly) || {!EGVAR(gear,enabled)}) exitWith {};
    params ["_source", "_target", "_unit"];
    private _plate = _target getVariable [QEGVAR(garage,plate), ""];
    private _fuelType = [_target] call FUNC(fuelType);
    EXTCALL("fuel:started",[ARR_5(netId _source,netId _target,_unit,_plate,_fuelType)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(tick), {
    if (EGVAR(gear,readOnly) || {!EGVAR(gear,enabled)}) exitWith {};
    params ["_source", "_target", "_amount"];
    EXTCALL("fuel:tick",[ARR_3(netId _source,netId _target,_amount)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(stopped), {
    if (EGVAR(gear,readOnly) || {!EGVAR(gear,enabled)}) exitWith {};
    params ["_source", "_target"];
    EXTCALL("fuel:stopped",[ARR_3(netId _source,netId _target,worldName)]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:fuel") exitWith {};
    if (_func != "price") exitWith {};
    [QGVAR(prices), [_data]] call CBA_fnc_globalEventJIP;
}];
