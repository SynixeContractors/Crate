#include "script_component.hpp"
ADDON = false;

["ace_refuel_started", {
    params ["_source", "_target", "", "_unit"];
    [QGVAR(started), [_source, _target, _unit getVariable [QEGVAR(discord,id), ""]]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;

["ace_refuel_tick", {
    params ["_source", "_target", "_amount"];
    [QGVAR(tick), [_source, _target, _amount]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;

["ace_refuel_finished", {
    params ["_source", "_target"];
    [QGVAR(finished), [_source, _target]] call CBA_fnc_serverEvent;
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

ADDON = true;
