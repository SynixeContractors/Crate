#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

[QGVAR(started), {
    params ["_source", "_target", "_unit"];
    private _plate = _target getVariable [QEGVAR(garage,plate), ""];
    EXTCALL("fuel:started",[ARR_4(netId _source, netId _target, _unit, _plate)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(tick), {
    params ["_source", "_target", "_amount"];
    EXTCALL("fuel:tick",[ARR_3(netId _source, netId _target, _amount)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(finished), {
    params ["_source", "_target"];
    EXTCALL("fuel:finished",[ARR_3(netId _source, netId _target, worldName)]);
}] call CBA_fnc_addEventHandler;
