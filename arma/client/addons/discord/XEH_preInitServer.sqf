#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

["ace_placedInBodyBag", {
    params ["_unit", "_bodybag"];
    private _discord = _unit getVariable [QGVAR(id), ""];
    if (_discord isEqualTo "") exitWith {};
    private _steam = _unit getVariable [QGVAR(steam), ""];
    if (_steam isEqualTo "") exitWith {};
    _bodybag setVariable [QGVAR(id), _discord, true];
    _bodybag setVariable [QGVAR(steam), _steam, true];
}] call CBA_fnc_addEventHandler;

[QGVAR(member), {
    params [
        ["_steam", "", [""]],
        ["_name", "", [""]]
    ];
    if (_steam == "") exitWith {};
    if (_name == "") exitWith {};
    EXTCALL("discord:member:get",[ARR_2(_steam,_name)]);
}] call CBA_fnc_addEventHandler;

[QGVAR(saveDLC), {
    params [
        ["_player", objNull, [objNull]],
        ["_dlc", [], [[]]]
    ];
    if (isNull _player) exitWith {};
    private _discord = _player getVariable [QGVAR(id), ""];
    if (_discord isEqualTo "") exitWith {};
    EXTCALL("discord:member:save_dlc",[ARR_2(_discord,_dlc)]);
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:discord") exitWith {};

    switch (_func) do {
        case "member:get:ok": {
            (parseSimpleArray _data) params ["_steam", "_discord", "_roles"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            if ((_player getVariable [QGVAR(id), ""]) != _discord) then {
                _player setVariable [QGVAR(id), _discord, true];
                [QGVAR(updatedId), [_discord], [_player]] call CBA_fnc_targetEvent;
            };
            if ((_player getVariable [QGVAR(roles), []]) isNotEqualTo _roles) then {
                _player setVariable [QGVAR(roles), _roles, true];
                [QGVAR(updatedRoles), [_roles], [_player]] call CBA_fnc_targetEvent;
            };
        };
        case "member:get:err": {
            (parseSimpleArray _data) params ["_steam"];
            serverCommand format ['#kick %1', _steam];
        };
        case "member:get:needs_link": {
            (parseSimpleArray _data) params ["_steam"];
            private _player = [_steam] call EFUNC(common,playerFromSteam);
            [QGVAR(needsLink), [], [_player]] call CBA_fnc_targetEvent;
            serverCommand format ['#kick %1', _steam];
        };
    };
}];
