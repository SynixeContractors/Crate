#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

[QGVAR(started), {
    if (EGVAR(gear,readOnly) || {!EGVAR(gear,enabled)}) exitWith {};
    params ["_source", "_target", "_unit"];
    private _plate = _target getVariable [QEGVAR(garage,plate), ""];
    EXTCALL("fuel:started",[ARR_4(netId _source,netId _target,_unit,_plate)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(tick), {
    if (EGVAR(gear,readOnly) || {!EGVAR(gear,enabled)}) exitWith {};
    params ["_source", "_target", "_amount"];
    EXTCALL("fuel:tick",[ARR_3(netId _source,netId _target,_amount)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(finished), {
    if (EGVAR(gear,readOnly) || {!EGVAR(gear,enabled)}) exitWith {};
    params ["_source", "_target"];
    EXTCALL("fuel:finished",[ARR_3(netId _source,netId _target,worldName)]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:fuel") exitWith {};
    if (_func != "price") exitWith {};
    GVAR(diaryRecord) = player createDiaryRecord [
        QEGVAR(discord,diary),
        [
            "Fuel Price",
            format ["The fuel price in this region is %1 per litre.", _data]
        ]
    ];
}];
