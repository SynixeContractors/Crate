#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

[QGVAR(started), {
    params ["_source", "_target", "_unit"];
    EXTCALL("fuel:started",[netId _source, netId _target, _unit, _target getVariable [QEGVAR(garage,plate), ""]]);
}] call CBA_fnc_addEventHandler;

[QGVAR(tick), {
    params ["_source", "_target", "_amount"];
    EXTCALL("fuel:tick",[netId _source, netId _target, _amount]);
}] call CBA_fnc_addEventHandler;

[QGVAR(finished), {
    params ["_source", "_target"];
    EXTCALL("fuel:finished",[netId _source, netId _target, worldName]);
}] call CBA_fnc_addEventHandler;
