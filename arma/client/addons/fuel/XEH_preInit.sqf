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

["ace_refuel_stopped", {
    params ["_source", "_target"];
    [QGVAR(stopped), [_source, _target]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;

[QGVAR(prices), {
    params ["_regular"];
    private _regular = parseNumber _regular;
    private _avgas = _regular * 1.6;
    private _jeta1 = _regular * 2.0;
    GVAR(diaryRecord) = player createDiaryRecord [
        QEGVAR(discord,diary),
        [
            "Fuel Prices",
            format ["The fuel prices in this region are:<br/>Regular: $%1 per litre<br/>Avgas: $%2 per litre<br/>Jet A-1: $%3 per litre",
                _regular,
                _avgas,
                _jeta1
            ]
        ]
    ];
}] call CBA_fnc_addEventHandler;

ADDON = true;
